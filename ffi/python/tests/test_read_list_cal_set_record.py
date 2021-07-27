import unittest
import os
from pathlib import Path
from citi import Record
import numpy.testing as npt


class TestReadListCalSetRecord(unittest.TestCase):

    @staticmethod
    def __get_display_memory_filename() -> str:
        relative_path = os.path.join('.', '..', '..', '..')
        this_dir = os.path.dirname(Path(__file__).absolute())
        absolute_path = os.path.join('tests', 'regression_files')
        filename = 'list_cal_set.cti'
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
        self.assertEqual(self.record.name, "CAL_SET")

    def test_comments(self):
        self.assertEqual(len(self.record.comments), 0)

    def test_devices(self):
        self.assertEqual(len(self.record.devices), 1)
        self.assertEqual(self.record.devices, [(
            "NA",
            [
                "VERSION HP8510B.05.00",
                "REGISTER 1",
                "SWEEP_TIME 9.999987E-2",
                "POWER1 1.0E1",
                "POWER2 1.0E1",
                "PARAMS 2",
                "CAL_TYPE 3",
                "POWER_SLOPE 0.0E0",
                "SLOPE_MODE 0",
                "TRIM_SWEEP 0",
                "SWEEP_MODE 4",
                "LOWPASS_FLAG -1",
                "FREQ_INFO 1",
                "SPAN 1000000000 3000000000 4",
                "DUPLICATES 0",
                "ARB_SEG 1000000000 1000000000 1",
                "ARB_SEG 2000000000 3000000000 3",
            ]
        )])

    def test_independent_variable(self):
        self.assertEqual(self.record.independent_variable[0], "FREQ")
        self.assertEqual(self.record.independent_variable[1], "MAG")
        npt.assert_array_almost_equal(
            self.record.independent_variable[2],
            [1000000000., 2000000000., 2500000000., 3000000000.]
        )

    def test_data(self):
        self.assertEqual(len(self.record.data), 3)

        self.assertEqual(self.record.data[0][0], 'E[1]')
        self.assertEqual(self.record.data[0][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[0][2],
            [
                (0.00173103+0.00173103j),
                (-0.00536775-0.00536775j),
                (0.0053265+0.0053265j),
                (-0.00407981-0.00407981j),
            ]
        )

        self.assertEqual(self.record.data[1][0], 'E[2]')
        self.assertEqual(self.record.data[1][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[1][2],
            [
                (-0.0082674-0.0082674j),
                (-0.0024871-0.0024871j),
                (-0.0306778-0.0306778j),
                (0.0599861+0.0599861j)
            ]
        )

        self.assertEqual(self.record.data[2][0], 'E[3]')
        self.assertEqual(self.record.data[2][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[2][2],
            [
                (0.431518+0.431518j),
                (-0.133056-0.133056j),
                (0.55841+0.55841j),
                (-0.807098-0.807098j)
            ]
        )
