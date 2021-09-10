import unittest
from citi import Record


class TestCannotFindFile(unittest.TestCase):

    def test_cannot_find_file(self):
        with self.assertRaises(NotImplementedError) as e:
            Record("filename that does not exist")

        self.assertEqual(str(e.exception), 'File not found for reading')
