from typing import Tuple

from authentik.lib.kerberos import iana


class EncryptionType:
    ENC_TYPE: iana.EncryptionType
    CHECKSUM_TYPE: iana.ChecksumType

    KEY_BYTES: int
    KEY_SEED_BYTES: int

    HMAC_BITS: int
    MESSAGE_BLOCK_BYTES: int

    CYPHER_BLOCK_BITS: int
    CONFOUNDER_BYTES: int

    DEFAULT_STRING_TO_KEY_PARAMS = ""

    HASH_FUNCTION = None

    @classmethod
    def random_to_key(cls, data: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def string_to_key(cls, password: str, salt: str, params: str) -> bytes:
        raise NotImplemented

    @classmethod
    def encrypt_data(cls, key: bytes, data: bytes) -> Tuple[bytes, bytes]:
        raise NotImplemented

    @classmethod
    def encrypt_message(cls, key: bytes, message: bytes, usage: int) -> Tuple[bytes, bytes]:
        raise NotImplemented

    @classmethod
    def decrypt_data(cls, key: bytes, data: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def decrypt_message(cls, key: bytes, ciphertext: bytes, usage: int) -> bytes:
        raise NotImplemented

    @classmethod
    def derive_key(cls, protocol_key: bytes, usage: bytes) -> bytes:
        raise NotImplemented

    @classmethod
    def derive_random(cls, protocol_key: bytes, usage: bytes) -> bytes:
        raise NotImplemented

    # TODO: verify_integrity(protocol_key: bytes, ciphertext: bytes, pt: bytes, usage: int) -> bool

    @classmethod
    def get_checksum_hash(cls, protocol_key: bytes, data: bytes, usage: int) -> bytes:
        raise NotImplemented

    @classmethod
    def verify_checksum(
        cls, protocol_key: bytes, data: bytes, checksum: bytes, usage: int
    ) -> bytes:
        raise NotImplemented
