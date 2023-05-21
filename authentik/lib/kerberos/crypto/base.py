from typing import Tuple

from cryptography.hazmat.primitives import hashes

from authentik.lib.kerberos import iana
from authentik.lib.kerberos.crypto.nfold import nfold


class EncryptionType:
    ENC_TYPE: iana.EncryptionType
    CHECKSUM_TYPE: iana.ChecksumType

    KEY_BYTES: int
    KEY_SEED_BITS: int

    HMAC_BITS: int
    MESSAGE_BLOCK_BYTES: int

    CYPHER_BLOCK_BITS: int
    CONFOUNDER_BYTES: int

    DEFAULT_STRING_TO_KEY_PARAMS = ""

    HASH_ALGORITHM: hashes.HashAlgorithm

    @classmethod
    def random_to_key(cls, data: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def derive_random(cls, key: bytes, usage: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def derive_key(cls, key: bytes, usage: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def string_to_key(cls, password: bytes, salt: bytes, params: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def encrypt_data(cls, key: bytes, data: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def encrypt_message(cls, key: bytes, message: bytes, usage: int) -> bytes:
        raise NotImplemented

    @classmethod
    def decrypt_data(cls, key: bytes, data: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def decrypt_message(cls, key: bytes, ciphertext: bytes, usage: int) -> bytes:
        raise NotImplemented

    # TODO: verify_integrity(key: bytes, ciphertext: bytes, pt: bytes, usage: int) -> bool

    @classmethod
    def get_checksum_hash(cls, key: bytes, data: bytes, usage: int) -> bytes:
        raise NotImplemented

    @classmethod
    def verify_checksum(cls, key: bytes, data: bytes, checksum: bytes, usage: int) -> bytes:
        raise NotImplemented


class Rfc3961(EncryptionType):
    """
    Encryption and checksuming method as defined in RFC 3961.

    See https://www.rfc-editor.org/rfc/rfc3961#section-6.3.1
    """

    @classmethod
    def random_to_key(cls, data: bytes) -> bytes:
        """
        Creates a protocol key from random bytes as defined in RFC 3961.

        See https://www.rfc-editor.org/rfc/rfc3961#section-6.3.1
        """
        if len(data) * 8 != cls.KEY_SEED_BITS:
            raise ValueError("invalid data length")

        def stretch_56_bits(data: bytes) -> bytes:
            def parity(b: int) -> Tuple[int, int]:
                """
                This is odd-parity.
                """
                lowest_bit = b & 1
                one_bits = 0
                # Count the number of set bits
                for i in range(1, 8):
                    if b & (1 << i):
                        one_bits += 1
                if one_bits % 2 == 0:
                    # Even number of 1 bits, parity is set as 1
                    b |= 1
                else:
                    # Odd number of 1 bits, parity is set as 0
                    b &= ~1
                return b, lowest_bit

            result = bytes()
            last_byte = 0
            for i in range(7):
                p, lowest_bit = parity(data[i])
                result += bytes([p])
                if lowest_bit == 1:
                    # If lowest bit was 1, set it in the last byte
                    # If it was 0, leave as is, as it's already 0
                    last_byte |= 1 << (i + 1)

            # Parity for last byte
            p, _ = parity(last_byte)
            result += bytes([p])

            return result

        result = bytes()
        for i in range(0, 21, 7):
            result += stretch_56_bits(data[i : i + 8])

        def fix_weak_keys(b: bytes) -> bytes:
            """
            See https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-67r2.pdf for a list of such keys.
            """
            weak_keys = [
                bytes(data * 3)
                for data in (
                    [0x01] * 8,
                    [0xFE] * 8,
                    [0xE0] * 4 + [0xF1] * 4,
                    [0x1F] * 4 + [0x0E] * 4,
                )
            ]
            semi_weak_keys = [
                bytes(data * 3)
                for data in (
                    [0x01, 0x1F, 0x01, 0x1F, 0x01, 0x0E, 0x01, 0x0E],
                    [0x1F, 0x01, 0x1F, 0x01, 0x0E, 0x01, 0x0E, 0x01],
                    [0x01, 0xE0, 0x01, 0xE0, 0x01, 0xF1, 0x01, 0xF1],
                    [0xE0, 0x01, 0xE0, 0x01, 0xF1, 0x01, 0xF1, 0x01],
                    [0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE],
                    [0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01],
                    [0x1F, 0xE0, 0x1F, 0xE0, 0x0E, 0xF1, 0x0E, 0xF1],
                    [0xE0, 0x1F, 0xE0, 0x1F, 0xF1, 0x0E, 0xF1, 0x0E],
                    [0x1F, 0xFE, 0x1F, 0xFE, 0x0E, 0xFE, 0x0E, 0xFE],
                    [0xFE, 0x1F, 0xFE, 0x1F, 0xFE, 0x0E, 0xFE, 0x0E],
                    [0xE0, 0xFE, 0xE0, 0xFE, 0xF1, 0xFE, 0xF1, 0xFE],
                    [0xFE, 0xE0, 0xFE, 0xE0, 0xFE, 0xF1, 0xFE, 0xF1],
                )
            ]

            if b in weak_keys or b in semi_weak_keys:
                return b[0:7] + bytes([b[7] ^ 0xF0])
            return b

        return fix_weak_keys(result)

    @classmethod
    def derive_random(cls, key: bytes, usage: bytes) -> bytes:
        """
        Derives random data from a protocol key and a usage.

        See https://www.rfc-editor.org/rfc/rfc3961#section-5.1
        """
        usage_folded = nfold(usage, cls.CYPHER_BLOCK_BITS // 8)
        data = cls.encrypt_data(key, usage_folded)
        result = bytes()
        while len(result) < cls.KEY_SEED_BITS // 8:
            result += data
            data = cls.encrypt_data(key, data)
        return result[: cls.KEY_SEED_BITS // 8]

    @classmethod
    def derive_key(cls, key: bytes, usage: bytes) -> bytes:
        """
        Derive data from a protocol key and a usage as defined in RFC 3961

        See https://www.rfc-editor.org/rfc/rfc3961#section-5.1
        """
        return cls.random_to_key(cls.derive_random(key, usage))

    @classmethod
    def string_to_key(cls, password: str, salt: str, params: str) -> bytes:
        """
        Creates a protocol key from a password and a salt as defined in RFC 3961.

        See https://www.rfc-editor.org/rfc/rfc3961#section-6.3.1
        """
        if params != "":
            raise ValueError("DES3 does not support any parameters")

        s = password + salt
        tmp_key = cls.random_to_key(nfold(s.encode(), cls.KEY_SEED_BITS // 8))
        return cls.derive_key(tmp_key, "kerberos".encode())


class Rfc3962(Rfc3961):
    @classmethod
    def random_to_key(cls, data: bytes) -> bytes:
        return data
