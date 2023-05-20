from django.test import TestCase

from authentik.lib.kerberos.crypto import Des3CbcHmacSha1Kd


class TestDes3CbcHmacSha1Kd(TestCase):
    def test_random_to_key(self):
        # Example from RFC 3961
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
