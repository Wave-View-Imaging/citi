import unittest
from citi import Record


class TestCannotFindFile(unittest.TestCase):

    def test_cannot_find_file(self):
        with self.assertRaises(NotImplementedError):
            Record("filename that does not exist")
