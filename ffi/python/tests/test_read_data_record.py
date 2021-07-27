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
                (-0.898651-0.898651j),
                (0.306915+0.306915j),
                (0.787323+0.787323j),
                (-0.705291-0.705291j),
                (-0.425537-0.425537j),
                (0.896606+0.896606j),
                (-0.110504-0.110504j),
                (-0.913787-0.913787j),
                (0.537841+0.537841j),
                (0.572082+0.572082j)
            ]
        )
