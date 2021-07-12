import unittest
from citi import Record


class TestDefaultRecord(unittest.TestCase):

    def setUp(self):
        self.record = Record()

    def test_version(self):
        self.assertEqual(self.record.version, "A.01.00")
