import unittest
from citi import Record


class TestRecordSetters(unittest.TestCase):

    def setUp(self):
        self.record = Record()

    def test_set_version(self):
        self.assertEqual(self.record.version, "A.01.00")
        self.record.version = "A.00.00"
        self.assertEqual(self.record.version, "A.00.00")
