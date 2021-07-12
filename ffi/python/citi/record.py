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
from ctypes import c_char_p, Structure, POINTER
from pathlib import Path


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
    A `ModuleNotFoundError` exception is raised is zero or more
    than one file matching the pattern is found.
    '''
    # Find the directory where the DLL exists
    dll_dir = os.path.dirname(Path(__file__).absolute())

    # Add pattern
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

# record_default
CITI_LIB.record_default.argtypes = ()
CITI_LIB.record_default.restype = POINTER(FFIRecord)

# record_destroy
CITI_LIB.record_destroy.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_destroy.restype = None

# record_get_version
CITI_LIB.record_get_version.argtypes = (POINTER(FFIRecord),)
CITI_LIB.record_get_version.restype = c_char_p


class Record():
    """Representation of a CITI file

    This is a C ABI FFI into an implementation written in Rust.
    """

    def __init__(self, filename=None):
        if filename is None:
            self.__obj = CITI_LIB.record_default()
        else:
            self.__obj = CITI_LIB.record_default()

    def __del__(self):
        CITI_LIB.record_destroy(self.__obj)

    @property
    def version(self) -> str:
        return CITI_LIB.record_get_version(self.__obj).decode("utf-8")