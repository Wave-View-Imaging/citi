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
                complex(1.12134E-3, 1.73103E-3),
                complex(4.23145E-3, -5.36775E-3),
                complex(-0.56815E-3, 5.32650E-3),
                complex(-1.85942E-3, -4.07981E-3)
            ]
        )

        self.assertEqual(self.record.data[1][0], 'E[2]')
        self.assertEqual(self.record.data[1][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[1][2],
            [
                complex(2.03895E-2, -0.82674E-2),
                complex(-4.21371E-2, -0.24871E-2),
                complex(0.21038E-2, -3.06778E-2),
                complex(1.20315E-2, 5.99861E-2)
            ]
        )

        self.assertEqual(self.record.data[2][0], 'E[3]')
        self.assertEqual(self.record.data[2][1], 'RI')
        npt.assert_array_almost_equal(
            self.record.data[2][2],
            [
                complex(4.45404E-1, 4.31518E-1),
                complex(8.34777E-1, -1.33056E-1),
                complex(-7.09137E-1, 5.58410E-1),
                complex(4.84252E-1, -8.07098E-1)
            ]
        )
