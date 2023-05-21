from django.test import TestCase

from authentik.lib.kerberos.crypto import Aes128CtsHmacSha196


class TestAes128CtsHmacSha196(TestCase):
    def test_string_to_key(self):
        """
        Test cases from RFC 3962, Appendix B.

        See https://www.rfc-editor.org/rfc/rfc3962#appendix-B
        """
        data = (
            # Iteration count, password, salt, resulting key
            (1, "password", "ATHENA.MIT.EDUraeburn".encode(), 0x42263C6E89F4FC28B8DF68EE09799F15),
            (2, "password", "ATHENA.MIT.EDUraeburn".encode(), 0xC651BF29E2300AC27FA469D693BDDA13),
            (
                1200,
                "password",
                "ATHENA.MIT.EDUraeburn".encode(),
                0x4C01CD46D632D01E6DBE230A01ED642A,
            ),
            (
                5,
                "password",
                0x1234567878563412.to_bytes(8, byteorder="big"),
                0xE9B23D52273747DD5C35CB55BE619D8E,
            ),
            (
                1200,
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
                "pass phrase equals block size".encode(),
                0x59D1BB789A828B1AA54EF9C2883F69ED,
            ),
            (
                1200,
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
                "pass phrase exceeds block size".encode(),
                0xCB8005DC5F90179A7F02104C0018751D,
            ),
            (
                50,
                "𝄞",
                "EXAMPLE.COMpianist".encode(),
                0xF149C1F2E154A73452D43E7FE62A56E5,
            ),
        )

        for iterations, password, salt, key in data:
            self.assertEqual(
                Aes128CtsHmacSha196.string_to_key(
                    password.encode(), salt, "{:08x}".format(iterations)
                ),
                key.to_bytes(16, byteorder="big"),
            )
