import unittest
from citi import Record


class TestErrorCodes(unittest.TestCase):

    def setUp(self) -> None:
        self.record = Record()

    def runner(self, code, expected) -> None:
        self.assertEqual(
            expected,
            self.record.get_error_description(code),
        )

    def test_non_existant_error_code(self):
        self.runner(1, 'Invalid error code')

    def test_non_existant_last_error_code(self):
        self.runner(-42, 'Invalid error code')

    def test_no_error(self):
        self.runner(0, 'No error')

    def test_unknown_error(self):
        self.runner(-1, 'Unknown error')

    def test_null_argument(self):
        self.runner(-2, 'Function argument is null')

    def test_invalid_utf8_string(self):
        self.runner(-3, 'Invalid UTF8 character found in string')

    def test_file_not_found(self):
        self.runner(-4, 'File not found for reading')

    def test_file_permission_denied(self):
        self.runner(-5, 'File permission denied for reading')

    def test_file_connection_refused(self):
        self.runner(-6, 'File connection refused for reading')

    def test_file_connection_reset(self):
        self.runner(-7, 'File connection reset while atttempting to read')

    def test_file_connection_aborted(self):
        self.runner(-8, 'File connection aborted while attempting to read')

    def test_file_not_connected(self):
        self.runner(-9, 'Connection to file failed while attempting to read')

    def test_file_addr_in_use(self):
        self.runner(-10, 'File address is already in use')
        
    def test_file_addr_not_available(self):
        self.runner(-11, 'File address is not available')

    def test_file_broken_pipe(self):
        self.runner(-12, 'Connection pipe for file is broken')

    def test_file_already_exists(self):
        self.runner(-13, 'File already exists')

    def test_file_world_block(self):
        self.runner(-14, 'File operation needs to block to complete')

    def test_file_invalid_input(self):
        self.runner(-15, 'Invalid input found for file operation')

    def test_file_invalid_data(self):
        self.runner(-16, 'Invalid data found during file operation')

    def test_file_timed_out(self):
        self.runner(-17, 'File operation timed out')

    def test_file_write_zero(self):
        self.runner(-18, 'File opertion could not be completed')

    def test_file_interrupted(self):
        self.runner(-19, 'File operation interrupted')

    def test_file_unexpected_eof(self):
        self.runner(-20, '`EOF` character was reached prematurely')

    def test_record_parse_error_bad_keyword(self):
        self.runner(-21, 'Keyword is not supported when parsing to record')

    def test_record_parse_error_bad_regex(self):
        self.runner(-22, 'Regular expression could not be parsed into record')

    def test_record_parse_error_number(self):
        self.runner(-23, 'Unable to parse number into record')

    def test_record_read_error_data_array_over_index(self):
        self.runner(-24, 'Record read error due to more data arrays than defined in header')

    def test_record_read_error_independent_variable_defined_twice(self):
        self.runner(-25, 'Record read error dude to independent variable defined twice')

    def test_record_read_error_single_use_keyword_defined_twice(self):
        self.runner(-26, 'Record read error due to single use keyword defined twice')

    def test_record_read_error_out_of_order_keyword(self):
        self.runner(-27, 'Record read error due to out of order keyword')

    def test_record_read_error_line_error(self):
        self.runner(-28, 'Record read error on line')

    def test_record_read_error_io(self):
        self.runner(-29, 'Record read error due to file IO')

    def test_record_read_error_no_version(self):
        self.runner(-30, 'Record read error due to undefined version')

    def test_record_read_error_no_name(self):
        self.runner(-31, 'Record read error due to undefined name')

    def test_record_read_error_no_independent_variable(self):
        self.runner(-32, 'Record read error due to undefined indepent variable')

    def test_record_read_error_no_data(self):
        self.runner(-33, 'Record read error due to undefined data name and format')

    def test_record_read_error_var_and_data_different_lengths(self):
        self.runner(-34, 'Record read error due to different lengths for independent variable and data array')

    def test_record_write_error_no_version(self):
        self.runner(-35, 'Record write error due to undefined version')

    def test_record_write_error_no_name(self):
        self.runner(-36, 'Record write error due to undefined name')

    def test_record_write_error_no_data_name(self):
        self.runner(-37, 'Record write error due to no name in one of data arrays')

    def test_record_write_error_no_data_format(self):
        self.runner(-38, 'Record write error due to no format in one of data arrays')

    def test_record_write_error_writting_error(self):
        self.runner(-39, 'Record write error due to file IO')

    def test_null_byte(self):
        self.runner(-40, 'An interior null byte was found in string')

    def test_index_out_of_bounds(self):
        self.runner(-41, 'Index is outside of acceptable bounds')
