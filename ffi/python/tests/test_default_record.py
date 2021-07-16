import unittest
from citi import Record


class TestDefaultRecord(unittest.TestCase):

    def setUp(self):
        self.record = Record()

    def test_version(self):
        self.assertEqual(self.record.version, "A.01.00")

    def test_name(self):
        self.assertEqual(self.record.name, "")

    def test_comments(self):
        self.assertEqual(len(self.record.comments), 0)
