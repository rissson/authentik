from typing import Tuple

from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC

from authentik.lib.kerberos import iana
from authentik.lib.kerberos.crypto.base import EncryptionType
from authentik.lib.kerberos.crypto.nfold import nfold


class Aes128CtsHmacSha196(EncryptionType):
    ENC_TYPE = iana.EncryptionType.AES128_CTS_HMAC_SHA1_96
    CHECKSUM_TYPE = iana.ChecksumType.HMAC_SHA1_96_AES128

    KEY_BYTES = 128 // 8
    KEY_SEED_BITS = 128

    HMAC_BITS = 96
    MESSAGE_BLOCK_BYTES = 16

    CYPHER_BLOCK_BITS = MESSAGE_BLOCK_BYTES * 8
    CONFOUNDER_BYTES = MESSAGE_BLOCK_BYTES

    DEFAULT_STRING_TO_KEY_PARAMS = "00001000"

    HASH_ALGORITHM = hashes.SHA1

    @classmethod
    def random_to_key(cls, data: bytes) -> bytes:
        return data

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
    def string_to_key(cls, password: bytes, salt: bytes, params: str) -> bytes:
        if len(params) != 8:
            raise ValueError("Invalid params length")
        iterations = int(params, base=16)

        kdf = PBKDF2HMAC(
            algorithm=cls.HASH_ALGORITHM(),
            length=cls.KEY_BYTES,
            salt=salt,
            iterations=iterations,
        )
        tmp_key = cls.random_to_key(kdf.derive(password))
        return cls.derive_key(tmp_key, "kerberos".encode())

    @classmethod
    def encrypt_data(cls, key: bytes, data: bytes) -> bytes:
        iv = bytes([0] * (cls.CYPHER_BLOCK_BITS // 8))
        cipher = Cipher(algorithms.AES128(key), modes.CBC(iv))
        encryptor = cipher.encryptor()
        return encryptor.update(data) + encryptor.finalize()
