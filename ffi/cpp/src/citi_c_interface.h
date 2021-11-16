/// This C header file is a mirror of the C interface from the Rust
/// side. Note that this file will need to be inluded with
/// `extern "C"`.

#ifndef CITI_C_H
#define CITI_C_H

#include <stdlib.h>

typedef void Record;

/// Get the last occured error code
///
/// This function retrieves the last saved error code;
/// error codes are either 0 indicating no error or
/// negative integral values. Note that this function
/// itself returns 0 if there was no saved error code.
int get_last_error_code();

/// Get string description for error code
///
/// This function should be called with the return value of
/// `get_last_error_code`.
const char* get_error_description(int error_code);

/// Free a pointer to `Record`
/// 
/// This can be called on `null`. After being freed, the pointer
/// is left dangling, still pointing to the freed memory. This
/// function returns an integer representation of the error code
/// to indicate whether the record was successfully destroyed.
int record_destroy(Record* record);

/// Create default record
/// 
/// This allocates memory and must be destroyed by the caller
/// (see [`record_destroy`]).
Record* record_default();

/// Read record from file
/// 
/// This allocates memory and must be destroyed by the caller
/// (see [`record_destroy`]).
/// - A null pointer is returned if the filename is null, a file corresponding
/// to the filename does not exist, or the file cannot be read
Record* record_read(const char* filename);

/// Write record to file
///
/// This function will write to a filepath the from the contents
/// of the given Record.
int record_write(Record* record, const char* filename);

/// Write record to buffer
///
/// This function will write to a buffer from the contents
/// of the given Record.
const char* record_serialize_to_string(Record* record);

/// Get the record version
/// 
/// - If the [`Record`] pointer is null, null is returned.
/// - If the current version cannot be cast to [`std::ffi::CString`], null is returned.
/// - Returned version in null terminated
const char* record_get_version(Record* record);

/// Set the record version
/// 
/// - If the [`Record`] pointer is null, the function does nothing and returns.
/// - If the version pointer is null, the function does nothing and returns.
/// - Input string should be UTF-8 encoded
int record_set_version(Record* record, const char* version);

/// Get the record name
/// 
/// - If the [`Record`] pointer is null, null is returned.
/// - If the current name cannot be cast to [`std::ffi::CString`], null is returned.
/// - Returned name in null terminated
const char* record_get_name(Record* record);

/// Set the record name
/// 
/// - If the [`Record`] pointer is null, the function does nothing and returns.
/// - If the name pointer is null, the function does nothing and returns.
/// - Input string should be UTF-8 encoded
int record_set_name(Record* record, const char* name);

/// Get the number of comments
/// 
/// - If the [`Record`] pointer is null, zero is returned.
int record_get_number_of_comments(Record* record);

/// Get an array of comments
/// 
/// - If the [`Record`] pointer is null, a null pointer is returned.
/// - If index is out of bounds, a null pointer is returned.
const char* record_get_comment(Record* record, size_t idx);

/// Append to an array of comments
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
int record_append_comment(Record* record, const char* comment);

/// Get the number of devices
/// 
/// - If the [`Record`] pointer is null, zero is returned.
int record_get_number_of_devices(Record* record);

/// Get the device name
/// 
/// - If the [`Record`] pointer is null, a null pointer is returned.
/// - If the index is out of bounds, a null pointer is returned.
const char* record_get_device_name(Record* record, size_t idx);

/// Append new device
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
int record_append_device(Record* record, const char* device_name);

/// Get the number of entries in a device
/// 
/// - If the [`Record`] pointer is null, zero.
/// - If the index is out of bounds, zero.
int record_get_device_number_of_entries(Record* record, size_t idx);

/// Get the entry from a device
/// 
/// - If the [`Record`] pointer is null, zero.
/// - If the index is out of bounds, zero.
const char* record_get_device_entry(Record* record, size_t device_idx, size_t entry_idx);
                
/// Append new entry to device
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
int record_append_entry_to_device(Record* record, size_t device_idx, const char* entry);

/// Get independent variable name
/// 
/// - If the [`Record`] pointer is null, return null pointer.
const char* record_get_independent_variable_name(Record* record);

/// Get independent variable format
/// 
/// - If the [`Record`] pointer is null, return null pointer.
const char* record_get_independent_variable_format(Record* record);

/// Get independent variable length
/// 
/// - If the [`Record`] pointer is null, return null pointer.
int record_get_independent_variable_length(Record* record);

/// Get independent variable array
/// 
/// - If the [`Record`] pointer is null, return null pointer.
const double* record_get_independent_variable_array(Record* record);

/// Set independent variable
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
int record_set_independent_variable(Record* record, const char* name, const char* format, const double* vals, size_t len);

/// Get number of data arrays
/// 
/// - If the [`Record`] pointer is null, return zero.
int record_get_number_of_data_arrays(Record* record);

/// Get data array name
/// 
/// - If the [`Record`] pointer is null, return null pointer.
/// - If the index is out of bounds, return null pointer.
const char* record_get_data_array_name(Record* record, size_t idx);

/// Get data array format
/// 
/// - If the [`Record`] pointer is null, return zero.
/// - If the index is out of bounds, return zero.
const char* record_get_data_array_format(Record* record, size_t idx);

/// Get data array length
/// 
/// - If the [`Record`] pointer is null, return zero.
int record_get_data_array_length(Record* record, size_t idx);

/// Get data array
/// 
/// - If the [`Record`] pointer is null, nothing happens.
/// - If the index is out of bounds, nothing happens.
/// - Caller is responsible for allocation of the appropriate size and deallocation.
int record_get_data_array(Record* record, size_t idx, double* real, double* imag);


/// Append data array
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
int record_append_data_array(
    Record* record, const char* name, const char* format,
    const double* reals, const double* imags, size_t len);

#endif
