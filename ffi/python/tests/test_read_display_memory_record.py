import unittest
import os
from pathlib import Path
from citi import Record
import numpy.testing as npt


class TestReadDisplayMemoryRecord(unittest.TestCase):

    @staticmethod
    def __get_display_memory_filename() -> str:
        relative_path = os.path.join('.', '..', '..', '..')
        this_dir = os.path.dirname(Path(__file__).absolute())
        absolute_path = os.path.join('tests', 'regression_files')
        filename = 'display_memory.cti'
        return os.path.join(
            this_dir, relative_path, absolute_path, filename
        )

    def setUp(self):
        self.record = Record(self.__get_display_memory_filename())

    def test_file_exists(self):
        os.path.isfile(self.__get_display_memory_filename())

    def test_version(self):
        self.assertEqual(self.record.version, "A.01.00")

    def test_name(self):
        self.assertEqual(self.record.name, "MEMORY")

    def test_comments(self):
        self.assertEqual(len(self.record.comments), 0)

    def test_devices(self):
        self.assertEqual(len(self.record.devices), 1)
        self.assertEqual(self.record.devices, [(
            "NA",
            ["VERSION HP8510B.05.00", "REGISTER 1"]
        )])

    def test_independent_variable(self):
        self.assertEqual(self.record.independent_variable, (
            "FREQ", "MAG", []
        ))

    def test_data(self):
        self.assertEqual(len(self.record.data), 1)
        self.assertEqual(self.record.data[0][0], 'S')
        self.assertEqual(self.record.data[0][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[0][2],
            [
                complex(-1.31189E-3, -1.47980E-3),
                complex(-3.67867E-3, -0.67782E-3),
                complex(-3.43990E-3, 0.58746E-3),
                complex(-2.70664E-4, -9.76175E-4),
                complex(0.65892E-4, -9.61571E-4),
            ]
        )
