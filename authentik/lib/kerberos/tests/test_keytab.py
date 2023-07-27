"""Kerberos Keytab tests"""
from datetime import datetime

from django.test import TestCase

from authentik.lib.kerberos import crypto, iana, keytab, principal


class TestKeytab(TestCase):
    """Kerberos Keytab tests"""

    def test_to_bytes(self):
        """Kerberos Keytab export test"""
        data = (
            # keytab to export, result
            (
                keytab.Keytab(
                    [
                        keytab.KeytabEntry(
                            principal=keytab.Principal(
                                realm="AUTHENTIK.COMPANY",
                                name=principal.PrincipalName(
                                    name_type=principal.PrincipalNameType.NT_PRINCIPAL,
                                    name=["user"],
                                ),
                            ),
                            timestamp=datetime(2023, 5, 22, 1, 8, 0),
                            key=keytab.EncryptionKey(
                                key_type=iana.EncryptionType.AES128_CTS_HMAC_SHA1_96,
                                key=crypto.Aes128CtsHmacSha196.string_to_key(
                                    "password".encode(),
                                    "b17700c3-230e-4f4b-97ec-bcab98f59e9e".encode(),
                                ),
                            ),
                            kvno=1,
                        )
                    ]
                ),
                bytes(
                    [
                        0x05,
                        0x02,
                        0x00,
                        0x00,
                        0x00,
                        0x3C,
                        0x00,
                        0x01,
                        0x00,
                        0x11,
                        0x41,
                        0x55,
                        0x54,
                        0x48,
                        0x45,
                        0x4E,
                        0x54,
                        0x49,
                        0x4B,
                        0x2E,
                        0x43,
                        0x4F,
                        0x4D,
                        0x50,
                        0x41,
                        0x4E,
                        0x59,
                        0x00,
                        0x04,
                        0x75,
                        0x73,
                        0x65,
                        0x72,
                        0x00,
                        0x00,
                        0x00,
                        0x01,
                        0x64,
                        0x6A,
                        0xC0,
                        0x70,
                        0x01,
                        0x00,
                        0x11,
                        0x00,
                        0x10,
                        0xF2,
                        0x73,
                        0x98,
                        0x60,
                        0x86,
                        0x76,
                        0xD7,
                        0x52,
                        0x0C,
                        0xFE,
                        0x52,
                        0xB3,
                        0x07,
                        0x0A,
                        0x60,
                        0xB6,
                        0x00,
                        0x00,
                        0x00,
                        0x01,
                    ]
                ),
            ),
            (
                keytab.Keytab(
                    [
                        keytab.KeytabEntry(
                            principal=keytab.Principal(
                                realm="EXAMPLE.ORG",
                                name=principal.PrincipalName(
                                    name_type=principal.PrincipalNameType.NT_SRV_HST,
                                    name=["http", "example.org"],
                                ),
                            ),
                            timestamp=datetime(2023, 5, 26, 20, 44, 40),
                            key=keytab.EncryptionKey(
                                key_type=enc_type.ENC_TYPE,
                                key=enc_type.string_to_key(
                                    "iw6ubo6quo9ahB0phueB2cuuKeeMaec2vea2theiqu6boe"
                                    "Daiguchoo5chai4Aix".encode(),
                                    "b03d4083-c0c0-4866-bda8-39b980588a9d".encode(),
                                ),
                            ),
                            kvno=3,
                        )
                        for enc_type in [
                            crypto.Aes128CtsHmacSha196,
                            crypto.Aes256CtsHmacSha196,
                            crypto.Aes128CtsHmacSha256128,
                            crypto.Aes256CtsHmacSha384192,
                        ]
                    ]
                ),
                bytes(
                    [
                        0x05,
                        0x02,
                        0x00,
                        0x00,
                        0x00,
                        0x43,
                        0x00,
                        0x02,
                        0x00,
                        0x0B,
                        0x45,
                        0x58,
                        0x41,
                        0x4D,
                        0x50,
                        0x4C,
                        0x45,
                        0x2E,
                        0x4F,
                        0x52,
                        0x47,
                        0x00,
                        0x04,
                        0x68,
                        0x74,
                        0x74,
                        0x70,
                        0x00,
                        0x0B,
                        0x65,
                        0x78,
                        0x61,
                        0x6D,
                        0x70,
                        0x6C,
                        0x65,
                        0x2E,
                        0x6F,
                        0x72,
                        0x67,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x64,
                        0x71,
                        0x1A,
                        0x38,
                        0x03,
                        0x00,
                        0x11,
                        0x00,
                        0x10,
                        0x04,
                        0x0E,
                        0xF6,
                        0xE2,
                        0x23,
                        0x4D,
                        0xC0,
                        0x3B,
                        0xBD,
                        0x00,
                        0xE7,
                        0x76,
                        0x1E,
                        0xD9,
                        0xA5,
                        0x21,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x00,
                        0x00,
                        0x00,
                        0x53,
                        0x00,
                        0x02,
                        0x00,
                        0x0B,
                        0x45,
                        0x58,
                        0x41,
                        0x4D,
                        0x50,
                        0x4C,
                        0x45,
                        0x2E,
                        0x4F,
                        0x52,
                        0x47,
                        0x00,
                        0x04,
                        0x68,
                        0x74,
                        0x74,
                        0x70,
                        0x00,
                        0x0B,
                        0x65,
                        0x78,
                        0x61,
                        0x6D,
                        0x70,
                        0x6C,
                        0x65,
                        0x2E,
                        0x6F,
                        0x72,
                        0x67,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x64,
                        0x71,
                        0x1A,
                        0x38,
                        0x03,
                        0x00,
                        0x12,
                        0x00,
                        0x20,
                        0x74,
                        0xEB,
                        0xA7,
                        0x96,
                        0x4C,
                        0xD3,
                        0x60,
                        0x9D,
                        0xE5,
                        0x87,
                        0xE0,
                        0x77,
                        0xFF,
                        0xF0,
                        0x07,
                        0x08,
                        0xCF,
                        0x7C,
                        0x38,
                        0xC1,
                        0x71,
                        0x24,
                        0xD4,
                        0x35,
                        0xA6,
                        0xE0,
                        0x6C,
                        0x9E,
                        0x0F,
                        0x31,
                        0xC9,
                        0xBD,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x00,
                        0x00,
                        0x00,
                        0x43,
                        0x00,
                        0x02,
                        0x00,
                        0x0B,
                        0x45,
                        0x58,
                        0x41,
                        0x4D,
                        0x50,
                        0x4C,
                        0x45,
                        0x2E,
                        0x4F,
                        0x52,
                        0x47,
                        0x00,
                        0x04,
                        0x68,
                        0x74,
                        0x74,
                        0x70,
                        0x00,
                        0x0B,
                        0x65,
                        0x78,
                        0x61,
                        0x6D,
                        0x70,
                        0x6C,
                        0x65,
                        0x2E,
                        0x6F,
                        0x72,
                        0x67,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x64,
                        0x71,
                        0x1A,
                        0x38,
                        0x03,
                        0x00,
                        0x13,
                        0x00,
                        0x10,
                        0x81,
                        0x12,
                        0x2E,
                        0x1A,
                        0x4E,
                        0x1A,
                        0xD6,
                        0x35,
                        0x39,
                        0x2D,
                        0xDB,
                        0xB8,
                        0x4E,
                        0xB0,
                        0xA3,
                        0x45,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x00,
                        0x00,
                        0x00,
                        0x53,
                        0x00,
                        0x02,
                        0x00,
                        0x0B,
                        0x45,
                        0x58,
                        0x41,
                        0x4D,
                        0x50,
                        0x4C,
                        0x45,
                        0x2E,
                        0x4F,
                        0x52,
                        0x47,
                        0x00,
                        0x04,
                        0x68,
                        0x74,
                        0x74,
                        0x70,
                        0x00,
                        0x0B,
                        0x65,
                        0x78,
                        0x61,
                        0x6D,
                        0x70,
                        0x6C,
                        0x65,
                        0x2E,
                        0x6F,
                        0x72,
                        0x67,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                        0x64,
                        0x71,
                        0x1A,
                        0x38,
                        0x03,
                        0x00,
                        0x14,
                        0x00,
                        0x20,
                        0xEA,
                        0xFD,
                        0x8A,
                        0x78,
                        0x8C,
                        0xDB,
                        0xE2,
                        0xC2,
                        0x02,
                        0x30,
                        0x09,
                        0xA8,
                        0x8E,
                        0x25,
                        0xB9,
                        0x51,
                        0xD0,
                        0x0B,
                        0xFE,
                        0x6D,
                        0x98,
                        0xF7,
                        0x5D,
                        0x9B,
                        0xED,
                        0x78,
                        0x97,
                        0x1C,
                        0xD8,
                        0x56,
                        0xE6,
                        0x1B,
                        0x00,
                        0x00,
                        0x00,
                        0x03,
                    ]
                ),
            ),
        )

        for kt, result in data:  # pylint: disable=invalid-name
            print(zip(kt.to_bytes(), result))
            self.assertEqual(kt.to_bytes(), result)
