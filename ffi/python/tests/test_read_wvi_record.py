import unittest
import os
from pathlib import Path
from citi import Record


class TestReadWVIRecord(unittest.TestCase):

    @staticmethod
    def __get_display_memory_filename() -> str:
        relative_path = os.path.join('.', '..', '..', '..')
        this_dir = os.path.dirname(Path(__file__).absolute())
        absolute_path = os.path.join('tests', 'regression_files')
        filename = 'wvi_file.cti'
        return os.path.join(
            this_dir, relative_path, absolute_path, filename
        )

    def setUp(self):
        self.record = Record(self.__get_display_memory_filename())

    def test_file_exists(self):
        os.path.isfile(self.__get_display_memory_filename())

    def test_version(self):
        self.assertEqual(self.record.version, "A.01.01")

    def test_name(self):
        self.assertEqual(self.record.name, "Antonly001")

    def test_comments(self):
        self.assertEqual(len(self.record.comments), 6)
        self.assertEqual(self.record.comments, [
            'SOURCE: 10095059066467',
            'DATE: Fri, Jan 18, 2019, 14:14:44',
            'ANTPOS_TX: 28.4E-3 0E+0 -16E-3 90 270 0',
            'ANTPOS_RX: 28.4E-3 0E+0 -16E-3 90 270 0',
            'ANT_TX: NAH_003',
            'ANT_RX: NAH_003',
        ])

    def test_devices(self):
        self.assertEqual(len(self.record.devices), 0)
