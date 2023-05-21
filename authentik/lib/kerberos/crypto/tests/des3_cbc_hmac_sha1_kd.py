from django.test import TestCase

from authentik.lib.kerberos.crypto import Des3CbcHmacSha1Kd


class TestDes3CbcHmacSha1Kd(TestCase):
    def test_random_to_key(self):
        """
        Test cases from RFC 3961
        """
        data = bytes(
            [
                0b11000000,
                0b00111100,
                0b11100011,
                0b01001000,
                0b10111001,
                0b00011011,
                0b00010111,
            ]
            * 3  # 3 times 56 bits, no point in doing it more
        )
        result = bytes(
            [
                0b11000001,
                0b00111101,
                0b11100011,
                0b01001001,
                0b10111001,
                0b00011010,
                0b00010110,
                0b11101001,
            ]
            * 3
        )

        self.assertEqual(Des3CbcHmacSha1Kd.random_to_key(data), result)

        with self.assertRaises(ValueError):
            Des3CbcHmacSha1Kd.random_to_key(bytes([0x42]))

    def test_derive_random_and_key(self):
        """
        Test cases from https://www.rfc-editor.org/rfc/rfc3961#appendix-A.3
        """
        cases = (
            # # key, result, derive_random output, derive_key output
            # (
            #     0xDCE06B1F64C857A11C3DB57C51899B2CC1791008CE973B92,
            #     0x0000000155,
            #     0x935079D14490A75C3093C4A6E8C3B049C71E6EE705,
            #     0x925179D04591A79B5D3192C4A7E9C289B049C71F6EE604CD,
            # ),
        )

        for key, usage, dr, dk in cases:
            self.assertEqual(
                Des3CbcHmacSha1Kd.derive_random(
                    key.to_bytes(24, byteorder="big"), usage.to_bytes(2, byteorder="big")
                ),
                dr.to_bytes(21),
            )
            # self.assertEqual(
            #     Des3CbcHmacSha1Kd.derive_key(
            #         key.to_bytes(24, byteorder="big"), usage.to_bytes(2, byteorder="big")
            #     ),
            #     dk.to_bytes(24, byteorder="big"),
            # )

    def test_string_to_key(self):
        pass
