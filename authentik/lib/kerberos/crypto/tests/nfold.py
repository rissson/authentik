from django.test import TestCase

from authentik.lib.kerberos.crypto.nfold import nfold


class TestNfold(TestCase):
    def test_nfold(self):
        """
        Test cases from https://www.rfc-editor.org/rfc/rfc3961#appendix-A.1
        """

        cases = (
            # data, size (in bits), output
            ("012345", 64, 0xBE072631276B1955),
            ("password", 56, 0x78A07B6CAF85FA),
            ("Rough Consensus, and Running Code", 64, 0xBB6ED30870B7F0E0),
            ("password", 168, 0x59E4A8CA7C0385C3C37B3F6D2000247CB6E6BD5B3E),
            (
                "MASSACHVSETTS INSTITVTE OF TECHNOLOGY",
                192,
                0xDB3B0D8F0B061E603282B308A50841229AD798FAB9540C1B,
            ),
            ("Q", 168, 0x518A54A215A8452A518A54A215A8452A518A54A215),
            ("ba", 168, 0xFB25D531AE8974499F52FD92EA9857C4BA24CF297E),
            # Additional with `kerberos` as data
            ("kerberos", 64, 0x6B65726265726F73),
            ("kerberos", 128, 0x6B65726265726F737B9B5B2B93132B93),
            ("kerberos", 168, 0x8372C236344E5F1550CD0747E15D62CA7A5A3BCEA4),
            ("kerberos", 256, 0x6B65726265726F737B9B5B2B93132B935C9BDCDAD95C9899C4CAE4DEE6D6CAE4),
        )

        for data, size, output in cases:
            self.assertEqual(
                nfold(data.encode(), size // 8), output.to_bytes(size // 8, byteorder="big")
            )
