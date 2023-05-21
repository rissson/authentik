from typing import Tuple

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes

from authentik.lib.kerberos import iana
from authentik.lib.kerberos.crypto.base import Rfc3961
from authentik.lib.kerberos.crypto.nfold import nfold


class Des3CbcHmacSha1Kd(Rfc3961):
    ENC_TYPE = iana.EncryptionType.DES3_CBC_SHA1_KD
    CHECKSUM_TYPE = iana.ChecksumType.HMAC_SHA1_DES3_KD

    KEY_BYTES = 24
    KEY_SEED_BITS = 21 * 8

    HMAC_BITS = 160
    MESSAGE_BLOCK_BYTES = 8

    CYPHER_BLOCK_BITS = 8 * 8
    CONFOUNDER_BYTES = 8

    DEFAULT_STRING_TO_KEY_PARAMS = ""

    HASH_FUNCTION = None

    @classmethod
    def encrypt_data(cls, key: bytes, data: bytes) -> bytes:
        """
        Encrypt raw data as defined in RFC 3961.

        See https://www.rfc-editor.org/rfc/rfc3961
        """
        cipher = Cipher(
            algorithms.TripleDES(key),
            # All bits at 0 for the initial cypher state, as per the RFC
            modes.CBC(bytes([0] * (cls.CYPHER_BLOCK_BITS // 8))),
        )
        encryptor = cipher.encryptor()
        ct = encryptor.update(data) + encryptor.finalize()
        # print("pt_in: ", data)
        # print("ct: ", ct)
        # decryptor = cipher.decryptor()
        # print("pt_out:", decryptor.update(ct) + decryptor.finalize())
        # return encryptor.update(data) + encryptor.finalize()
        return ct
