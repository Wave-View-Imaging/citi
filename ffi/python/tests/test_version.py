import unittest
import citi


class TestVersion(unittest.TestCase):

    def test_version(self):
        self.assertTrue(hasattr(citi, '__version__'))
