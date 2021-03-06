'''Record implemented through a foreign function interface

A dynamic library is compiled in Rust to a C ABI. Interfaces
are broken out from there. The general idea is to keep the data
on the Rust side, store a pointer to the data on the Python side
and access that data through functions defined in Rust.

The dynamic library is automatically loaded upon import. A
ModuleNotFoundError is thrown if the dynamic library cannot
be found.
'''

import ctypes
import glob
import os
import sys
from ctypes import c_char_p, Structure, POINTER, c_size_t, c_double, c_int
from pathlib import Path
from typing import List, Union, Optional


def __get_library_name() -> str:
    '''Get the path to the DLL created by rust

    This is really a hack, but we're generally looking for a file
    of the name style:
        site-packages/citi/citi.cpython-38-darwin.so
    I could not find how this file is named by setuptools-rust.
    However, this is a nice platform independent hack to get that
    DLL so we can load it.

    However, no other files can match the glob pattern:
        *citi*cpython*
    A `ModuleNotFoundError` exception is raised if zero or more
    than one file matching the pattern is found.
    '''
    # Find the directory where the DLL exists
    dll_dir = os.path.dirname(Path(__file__).absolute())

    # Add pattern
    if sys.platform.startswith('win'):
        pattern = f'*citi*{sys.version_info[0]}{sys.version_info[1]}*.pyd'
    else:
        pattern = f'*citi*cpython*{sys.version_info[0]}{sys.version_info[1]}*'
    dll_pattern = os.path.join(dll_dir, pattern)

    # Find the DLL
    filename = glob.glob(dll_pattern)

    # Return if one found, otherwise empty
    if len(filename) == 1:
        return filename[0]
    else:
        raise ModuleNotFoundError('Could not find the citi dynamic library')


class FFIRecord(Structure):
    '''Python representation of the Rust class.

    This is used for holding and passing a pointer
    between Python and Rust.'''
    pass


# Hold the DLL interface
# Following this, the FFI functions are given argument
# and result types for ease of use. These types match the
# defined functions in Rust.
CITI_LIB = ctypes.CDLL(__get_library_name())

# get_last_error_code
CITI_LIB.get_last_error_code.argtypes = ()
CITI_LIB.get_last_error_code.restype = c_int

# get_error_description
CITI_LIB.get_error_description.argtypes = (c_int,)
CITI_LIB.get_error_description.restype = c_char_p

# record_default
CITI_LIB.record_default.argtypes = ()
CITI_LIB.record_default.restype = POINTER(FFIRecord)

# record_read
CITI_LIB.record_read.argtypes = (c_char_p,)
CITI_LIB.record_read.restype = POINTER(FFIRecord)

# record_destroy
CITI_LIB.record_destroy.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_destroy.restype = None

# record_get_version
CITI_LIB.record_get_version.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_version.restype = c_char_p

# record_set_version
CITI_LIB.record_set_version.argtypes = (POINTER(FFIRecord), c_char_p)
CITI_LIB.record_set_version.restype = None

# record_get_name
CITI_LIB.record_get_name.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_name.restype = c_char_p

# record_set_name
CITI_LIB.record_set_name.argtypes = (POINTER(FFIRecord), c_char_p)
CITI_LIB.record_set_name.restype = None

# record_get_number_of_comments
CITI_LIB.record_get_number_of_comments.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_number_of_comments.restype = c_size_t

# record_get_comment
CITI_LIB.record_get_comment.argtypes = (POINTER(FFIRecord), c_size_t)
CITI_LIB.record_get_comment.restype = c_char_p

# record_get_number_of_devices
CITI_LIB.record_get_number_of_devices.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_number_of_devices.restype = c_size_t

# record_get_device_name
CITI_LIB.record_get_device_name.argtypes = (POINTER(FFIRecord), c_size_t)
CITI_LIB.record_get_device_name.restype = c_char_p

# record_get_device_number_of_entries
CITI_LIB.record_get_device_number_of_entries.argtypes = \
    (POINTER(FFIRecord), c_size_t)
CITI_LIB.record_get_device_number_of_entries.restype = c_size_t

# record_get_device_entry
CITI_LIB.record_get_device_entry.argtypes = \
    (POINTER(FFIRecord), c_size_t, c_size_t)
CITI_LIB.record_get_device_entry.restype = c_char_p

# record_get_independent_variable_name
CITI_LIB.record_get_independent_variable_name.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_independent_variable_name.restype = c_char_p

# record_get_independent_variable_format
CITI_LIB.record_get_independent_variable_format.argtypes = \
    (POINTER(FFIRecord),)
CITI_LIB.record_get_independent_variable_format.restype = c_char_p

# record_get_independent_variable_length
CITI_LIB.record_get_independent_variable_length.argtypes = \
    (POINTER(FFIRecord),)
CITI_LIB.record_get_independent_variable_length.restype = c_size_t

# record_get_independent_variable_array
CITI_LIB.record_get_independent_variable_array.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_independent_variable_array.restype = POINTER(c_double)

# record_get_number_of_data_arrays
CITI_LIB.record_get_number_of_data_arrays.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_number_of_data_arrays.restype = c_size_t

# record_get_data_array_name
CITI_LIB.record_get_data_array_name.argtypes = (POINTER(FFIRecord), c_size_t)
CITI_LIB.record_get_data_array_name.restype = c_char_p

# record_get_data_array_format
CITI_LIB.record_get_data_array_format.argtypes = (POINTER(FFIRecord), c_size_t)
CITI_LIB.record_get_data_array_format.restype = c_char_p

# record_get_data_array_length
CITI_LIB.record_get_data_array_length.argtypes = (POINTER(FFIRecord), c_size_t)
CITI_LIB.record_get_data_array_length.restype = c_size_t

# record_get_data_array
CITI_LIB.record_get_data_array.argtypes = \
    (POINTER(FFIRecord), c_size_t, POINTER(c_double), POINTER(c_double))
CITI_LIB.record_get_data_array.restype = None


class Record():
    """Representation of a CITI file

    This is a C ABI FFI into an implementation written in Rust.
    """

    def __init__(self, filename: Optional[str] = None):
        # Get pointer to object
        if filename is None:
            self.__obj = CITI_LIB.record_default()
        else:
            self.__obj = CITI_LIB.record_read(filename.encode('utf-8'))

        if not self.__obj:
            error_code = self.last_error_code()
            if error_code != 0:
                raise NotImplementedError(
                    self.get_error_description(error_code)
                )
            else:
                raise NotImplementedError('A null pointer was returned')

    def __del__(self):
        # Can free null
        CITI_LIB.record_destroy(self.__obj)

    def last_error_code(self) -> int:
        return int(CITI_LIB.get_last_error_code())

    def get_error_description(self, error_code: int) -> str:
        return CITI_LIB.get_error_description(error_code).decode("utf-8")

    @property
    def version(self) -> str:
        '''Get the version string'''
        return CITI_LIB.record_get_version(self.__obj).decode("utf-8")

    @version.setter
    def version(self, value: str):
        '''Set the version string'''
        CITI_LIB.record_set_version(self.__obj, value.encode('utf-8'))

    @property
    def name(self) -> str:
        '''Get the record name'''
        return CITI_LIB.record_get_name(self.__obj).decode("utf-8")

    @name.setter
    def name(self, value: str):
        '''Set the record name'''
        CITI_LIB.record_set_name(self.__obj, value.encode('utf-8'))

    @property
    def comments(self) -> List[str]:
        '''Get the comments'''
        comments = []

        for i in range(CITI_LIB.record_get_number_of_comments(self.__obj)):
            comments.append(
                CITI_LIB.record_get_comment(
                    self.__obj, ctypes.c_size_t(i)
                ).decode('utf-8')
            )

        return comments

    @property
    def devices(self) -> List[Union[str, List[str]]]:
        '''Get the devices'''
        devices = []

        for d in range(CITI_LIB.record_get_number_of_devices(self.__obj)):
            name = CITI_LIB.record_get_device_name(
                self.__obj, ctypes.c_size_t(d)
            ).decode('utf-8')

            entries = []
            for e in range(CITI_LIB.record_get_device_number_of_entries(
                    self.__obj, ctypes.c_size_t(d))):
                entries.append(
                    CITI_LIB.record_get_device_entry(
                        self.__obj, ctypes.c_size_t(d), ctypes.c_size_t(e)
                    ).decode('utf-8')
                )

            devices.append((name, entries))

        return devices

    @property
    def independent_variable(self) -> Union[str, str, List[float]]:
        '''Get the independent variable

        A tuple is returned that is formatted:
            (Name: str, Format: str, independent_variable: List[float])
        '''
        name = CITI_LIB.record_get_independent_variable_name(self.__obj)\
            .decode('utf-8')
        format = CITI_LIB.record_get_independent_variable_format(self.__obj)\
            .decode('utf-8')
        len = CITI_LIB.record_get_independent_variable_length(self.__obj)
        array = CITI_LIB.record_get_independent_variable_array(self.__obj)

        iv = []
        for i in range(len):
            iv.append(array[i])

        return (name, format, iv)

    @property
    def data(self) -> List[Union[str, str, List[complex]]]:
        '''Get the data arrays

        A list of tuples is returned that are formatted:
            [(Name: str, Format: str, independent_variable: List[Complex])]
        '''
        data = []
        for i in range(CITI_LIB.record_get_number_of_data_arrays(self.__obj)):
            # Read names
            name = CITI_LIB.record_get_data_array_name(
                self.__obj, ctypes.c_size_t(i)
            ).decode('utf-8')

            format = CITI_LIB.record_get_data_array_format(
                self.__obj, ctypes.c_size_t(i)
            ).decode('utf-8')

            # Read arrays
            data_length = CITI_LIB.record_get_data_array_length(
                self.__obj, ctypes.c_size_t(i)
            )
            array = []
            real_ptr = (c_double * data_length)()
            imag_ptr = (c_double * data_length)()
            CITI_LIB.record_get_data_array(
                self.__obj, ctypes.c_size_t(i), real_ptr, imag_ptr
            )
            for d in range(data_length):
                array.append(complex(real_ptr[d], imag_ptr[d]))

            data.append((name, format, array))

        return data
