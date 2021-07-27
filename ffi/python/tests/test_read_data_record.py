import unittest
import os
from pathlib import Path
from citi import Record
import numpy.testing as npt
import numpy as np


class TestReadDataRecord(unittest.TestCase):

    @staticmethod
    def __get_display_memory_filename() -> str:
        relative_path = os.path.join('.', '..', '..', '..')
        this_dir = os.path.dirname(Path(__file__).absolute())
        absolute_path = os.path.join('tests', 'regression_files')
        filename = 'data_file.cti'
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
        self.assertEqual(self.record.name, "DATA")

    def test_comments(self):
        self.assertEqual(len(self.record.comments), 0)

    def test_devices(self):
        self.assertEqual(len(self.record.devices), 1)
        self.assertEqual(self.record.devices, [(
            "NA",
            ["VERSION HP8510B.05.00", "REGISTER 1"]
        )])

    def test_independent_variable(self):
        self.assertEqual(self.record.independent_variable[0], "FREQ")
        self.assertEqual(self.record.independent_variable[1], "MAG")
        npt.assert_array_almost_equal(
            self.record.independent_variable[2],
            np.linspace(1000000000., 4000000000., 10)
        )

    def test_data(self):
        self.assertEqual(len(self.record.data), 1)
        self.assertEqual(self.record.data[0][0], 'S[1,1]')
        self.assertEqual(self.record.data[0][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[0][2],
            [
                complex(0.86303E-1, -8.98651E-1),
                complex(8.97491E-1, 3.06915E-1),
                complex(-4.96887E-1, 7.87323E-1),
                complex(-5.65338E-1, -7.05291E-1),
                complex(8.94287E-1, -4.25537E-1),
                complex(1.77551E-1, 8.96606E-1),
                complex(-9.35028E-1, -1.10504E-1),
                complex(3.69079E-1, -9.13787E-1),
                complex(7.80120E-1, 5.37841E-1),
                complex(-7.78350E-1, 5.72082E-1),
            ]
        )
