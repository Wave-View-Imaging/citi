//! Foreign Function Interface
//! 
//! This FFI was designed to work as follows:
//! - Rust creates and destorys a pointer to a [Record]
//! - The interfacing language holds the pointer and is
//! responsible for creation, destruction, and avoiding
//! memory leaks.
//! - Rust methods take the pointer and modify the pointer
//! or return a value based on the interface.
//!
//! Error handling
//!
//! All errors are indicated using an error code represented by
//! a negative integral type. A string description of the the
//! error code can be retrieved using the `get_error_description`
//! function. Note that all functions as part of the C-api will either
//! return a pointer (null pointers represent an error) or an integer
//! where negative values represent an error code.

use crate::{Record, DataArray, Device, Error, ParseError, ReadError, WriteError};

use num_complex::Complex;
use std::ffi::{CString, CStr};
use libc::{c_char, c_double, c_int, size_t};
use std::fs::File;
use std::cell::RefCell;

/// Error code values must be maintained across any ffi boundaries
#[derive(Copy, Clone, PartialEq)]
enum ErrorCode {
    NoError = 0,
    UnknownError = -1,

    NullArgument = -3,

    //CStr::from_ptr
    InvalidUTF8String = -4,

    // File::open, File::create
    FileNotFound = -5,
    FilePermissionDenied = -6,
    FileConnectionRefused = -7,
    FileConnectionReset = -8,
    FileConnectionAborted = -9,
    FileNotConnected = -10,
    FileAddrInUse = -11,
    FileAddrNotAvailable = -12,
    FileBrokenPipe = -13,
    FileAlreadyExists = -14,
    FileWouldBlock = -15,
    FileInvalidInput = -16,
    FileInvalidData = -17,
    FileTimedOut = -18,
    FileWriteZero = -19,
    FileInterrupted = -20,
    FileUnexpectedEof = -21,

    // Record::from_reader
    // Record parse errors
    RecordParseErrorBadKeyword = -22,
    RecordParseErrorBadRegex = -23,
    RecordParseErrorNumber = -24,

    // Record read errors
    RecordReadErrorDataArrayOverIndex = -25,
    RecordReadErrorIndependentVariableDefinedTwice = -26,
    RecordReadErrorSingleUseKeywordDefinedTwice = -27,
    RecordReadErrorOutOfOrderKeyword = -28,
    RecordReadErrorLineError = -29,
    RecordReadErrorIO = -30,
    RecordReadErrorNoVersion = -31,
    RecordReadErrorNoName = -32,
    RecordReadErrorNoIndependentVariable = -33,
    RecordReadErrorNoData = -34,
    RecordReadErrorVarAndDataDifferentLengths = -35,

    // Record write errors
    RecordWriteErrorNoVersion = -36,
    RecordWriteErrorNoName = -37,
    RecordWriteErrorNoDataName = -38,
    RecordWriteErrorNoDataFormat = -39,
    RecordWriteErrorWrittingError = -40,

    // CString::new
    NullByte = -41,

    IndexOutOfBounds = -42
}

/// Note that this static array must be kept in sync with the error code enum.
const ERROR_DESCRIPTION: &[&str] = &[
    "No error",

    "Function argument is null",

    //CStr::from_ptr
    "Invalid UTF8 character found in string",
    
    // File::open
    "File not found for reading",
    "File permission denied for reading",
    "File connection refused for reading",
    "File connection reset while atttempting to read",
    "File connection aborted while attempting to read",
    "Connection to file failed while attempting to read",
    "File address is already in use",
    "File address is not available",
    "Connection pipe for file is broken",
    "File already exists",
    "File operation needs to block to complete",
    "Invalid input found for file operation",
    "Invalid data found during file operation",
    "File operation timed out",
    "File opertion could not be completed",
    "File operation interrupted",
    "`EOF` character was reached prematurely",
    "File operation is unsupported",
    "File operation failed due to insufficient memory",

    // Record::from_reader
    // ParseError descriptions
    "Keyword is not supported when parsing to record",
    "Regular expression could not be parsed into record",
    "Unable to parse number into record",

    // ReadError descriptions
    "Record read error due to more data arrays than defined in header",
    "Record read error dude to independent variable defined twice",
    "Record read error due to single use keyword defined twice",
    "Record read error due to out of order keyword",
    "Record read error on line",
    "Record read error due to file IO",
    "Record read error due to undefined version",
    "Record read error due to undefined name",
    "Record read error due to undefined indepent variable",
    "Record read error due to undefined data name and format",
    "Record read error due to different lengths for independent variable and data array",

    "Record write error due to undefined version",
    "Record write error due to undefined name",
    "Record write error due to no name in one of data arrays",
    "Record write error due to no format in one of data arrays",
    "Record write error due to file IO",

    // CString::new
    "An interior null byte was found in string",

    "Index is outside of acceptable bounds",
];

thread_local!{

    /// Node that this error code enum must be kept in sync with the 
    /// static array corresponding to error descriptions.

    static LAST_ERROR_CODE: RefCell<Option<ErrorCode>> = RefCell::new(None);
}

/// Update the last saved error code
///
/// This function is not public and is for use from the rust
/// side to update the error code.
fn update_error_code(error_code: ErrorCode) -> ErrorCode {
    LAST_ERROR_CODE.with(|prev| {
        prev.replace(Some(error_code))
    });

    error_code
}

/// Get the last occured error code
///
/// This function retrieves the last saved error code;
/// error codes are either 0 indicating no error or
/// negative integral values. Note that this function
/// itself returns 0 if there was no saved error code.
#[no_mangle]
pub extern "C" fn get_last_error_code() -> c_int {
    let last_error_code = LAST_ERROR_CODE.with(|prev| {
        prev.borrow_mut().take() 
    });

    if let Some(error_code) = last_error_code {
        return error_code as c_int;
    }

    // Return zero to indicate that there was no error
    0
}

/// Get string description for error code
///
/// This function should be called with the return value of
/// `get_last_error_code`.
#[no_mangle]
pub extern "C" fn get_error_description(error_code: c_int) -> *const c_char {
    // Convert error code to index into static description array
    let error_description_index = -error_code;

    if error_description_index < 0 || error_description_index >= ERROR_DESCRIPTION.len() as c_int {
        return unsafe { CStr::from_bytes_with_nul_unchecked(b"Invalid error code\0").as_ptr() };
    }
    
    // This should never return an error type
    let c_str = CString::new(ERROR_DESCRIPTION[error_description_index as usize]).unwrap();

    c_str.into_raw()
}

/// Check if index is out of bounds
fn check_index_bounds(idx: size_t, size: size_t) -> bool {

    if idx >= size {
        return false
    }

    return true
}

/// Helper function to validate pointers and update data field
fn set_str_value<T>(ptr: *mut T, val: *const c_char, update_field: fn(&mut T, str_val: String)) -> ErrorCode {

    if ptr.is_null() {
        return update_error_code(ErrorCode::NullArgument)
    }

    if val.is_null() {
        return update_error_code(ErrorCode::NullArgument)
    }

    let str_val = match unsafe { CStr::from_ptr(val) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid utf8 encoding
            return ErrorCode::InvalidUTF8String
        }
    };

    update_field(unsafe { &mut *ptr }, str_val);

    ErrorCode::NoError
}

/// Helper function to validate pointers and get value from data field
fn check_ptr_transform_to_cstring(record: *const Record, get_val: fn(&Record) -> &str) -> *const c_char {

    if record.is_null() {
        update_error_code(ErrorCode::NullArgument);
        return std::ptr::null_mut()
    }

    let record_ref = unsafe { &*record };

    // Convert to C string. Going through CString adds null terminator.
    let c_str = match CString::new(get_val(record_ref)) {
        Ok(s) => s,
        Err(_) => {
            // The only expected error is due to an interior null byte
            update_error_code(ErrorCode::NullByte);
            return std::ptr::null_mut()
        }
    };

    c_str.into_raw()
}

/// Helper function to validate pointers and get value from index
fn check_ptr_index_transform_to_cstring<T>(
    record: *const Record, 
    idx: size_t, 
    get_vals: impl Fn(&Record) -> &[T], 
    get_val: impl Fn(&[T], size_t) -> &str) -> *const c_char {

    if record.is_null() {
        update_error_code(ErrorCode::NullArgument);
        return std::ptr::null_mut()
    }

    let record_ref = unsafe { &*record };
    let vals = get_vals(record_ref);

    if check_index_bounds(idx, vals.len()) == false {
        update_error_code(ErrorCode::IndexOutOfBounds);
        return std::ptr::null_mut()
    }

    let val = get_val(vals, idx);

    // Convert to C string. Going through CString adds null terminator.
    let c_str = match CString::new(val) {
        Ok(s) => s,
        Err(_) => {
            // The only expected error is due to an interior null byte
            update_error_code(ErrorCode::NullByte);
            return std::ptr::null_mut()
        }
    };

    c_str.into_raw()
}

/// Helper function to map rust error type to ErrorCode
fn map_io_error_to_error_code(err: std::io::Error) -> ErrorCode {
    match err.kind() {
        std::io::ErrorKind::NotFound => update_error_code(ErrorCode::FileNotFound),
        std::io::ErrorKind::PermissionDenied => update_error_code(ErrorCode::FilePermissionDenied),
        std::io::ErrorKind::ConnectionRefused => update_error_code(ErrorCode::FileConnectionRefused),
        std::io::ErrorKind::ConnectionReset => update_error_code(ErrorCode::FileConnectionReset),
        std::io::ErrorKind::ConnectionAborted => update_error_code(ErrorCode::FileConnectionAborted),
        std::io::ErrorKind::NotConnected => update_error_code(ErrorCode::FileNotConnected),
        std::io::ErrorKind::AddrInUse => update_error_code(ErrorCode::FileAddrInUse),
        std::io::ErrorKind::AddrNotAvailable => update_error_code(ErrorCode::FileAddrNotAvailable),
        std::io::ErrorKind::BrokenPipe => update_error_code(ErrorCode::FileBrokenPipe),
        std::io::ErrorKind::AlreadyExists => update_error_code(ErrorCode::FileAlreadyExists),
        std::io::ErrorKind::WouldBlock => update_error_code(ErrorCode::FileWouldBlock),
        std::io::ErrorKind::InvalidInput => update_error_code(ErrorCode::FileInvalidInput),
        std::io::ErrorKind::InvalidData => update_error_code(ErrorCode::FileInvalidData),
        std::io::ErrorKind::TimedOut => update_error_code(ErrorCode::FileTimedOut),
        std::io::ErrorKind::WriteZero => update_error_code(ErrorCode::FileWriteZero),
        std::io::ErrorKind::Interrupted => update_error_code(ErrorCode::FileInterrupted),
        std::io::ErrorKind::UnexpectedEof => update_error_code(ErrorCode::FileUnexpectedEof),
        // Based on the docs, it is not recommended to match an error against `Other,
        _ => update_error_code(ErrorCode::UnknownError),
    }
}

/// Helper function to map rust error type to ErrorCode
fn map_record_error_to_error_code(err: Error) -> ErrorCode {
    match err {
        Error::ParseError(parse_err) => {
            match parse_err {
                ParseError::BadKeyword(_) => update_error_code(ErrorCode::RecordParseErrorBadKeyword),
                ParseError::BadRegex => update_error_code(ErrorCode::RecordParseErrorBadRegex),
                ParseError::NumberParseError(_) => update_error_code(ErrorCode::RecordParseErrorNumber)
            }
        },
        Error::ReadError(read_err) => {
            match read_err {
                ReadError::DataArrayOverIndex => update_error_code(ErrorCode::RecordReadErrorDataArrayOverIndex),
                ReadError::IndependentVariableDefinedTwice => update_error_code(ErrorCode::RecordReadErrorIndependentVariableDefinedTwice),
                ReadError::SingleUseKeywordDefinedTwice(_) => update_error_code(ErrorCode::RecordReadErrorSingleUseKeywordDefinedTwice),
                ReadError::OutOfOrderKeyword(_) => update_error_code(ErrorCode::RecordReadErrorOutOfOrderKeyword),
                ReadError::LineError(_, _) => update_error_code(ErrorCode::RecordReadErrorLineError),
                ReadError::ReadingError(_) => update_error_code(ErrorCode::RecordReadErrorIO),
                ReadError::NoVersion => update_error_code(ErrorCode::RecordReadErrorNoVersion),
                ReadError::NoName => update_error_code(ErrorCode::RecordReadErrorNoName),
                ReadError::NoIndependentVariable => update_error_code(ErrorCode::RecordReadErrorNoIndependentVariable),
                ReadError::NoData => update_error_code(ErrorCode::RecordReadErrorNoData),
                ReadError::VarAndDataDifferentLengths(_, _, _) => update_error_code(ErrorCode::RecordReadErrorVarAndDataDifferentLengths),
            }
        },
        Error::WriteError(write_err) => {
            match write_err {
                WriteError::NoVersion => update_error_code(ErrorCode::RecordWriteErrorNoVersion),
                WriteError::NoName => update_error_code(ErrorCode::RecordWriteErrorNoName),
                WriteError::NoDataName(_) => update_error_code(ErrorCode::RecordWriteErrorNoDataName),
                WriteError::NoDataFormat(_) => update_error_code(ErrorCode::RecordWriteErrorNoDataFormat),
                WriteError::WrittingError(_) => update_error_code(ErrorCode::RecordWriteErrorWrittingError),
            }
        }
    }
}

/// Free a pointer to `Record`
/// 
/// This can be called on `null`. After being freed, the pointer
/// is left dangling, still pointing to the freed memory. This
/// function returns an integer representation of the error code
/// to indicate whether the record was successfully destroyed.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_destroy(record: *mut Record) -> c_int {
    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    unsafe { drop(Box::from_raw(record)) }

    update_error_code(ErrorCode::NoError) as c_int
}

#[cfg(test)]
mod destory {
    use super::*;

    #[test]
    fn destory_null() {
        let record_ptr: *mut Record = std::ptr::null_mut();
        record_destroy(record_ptr);
        assert!(record_ptr.is_null());
    }

    #[test]
    fn destory_not_null() {
        let record_ptr = Box::into_raw(Box::new(Record::default()));
        record_destroy(record_ptr);
        assert!(!record_ptr.is_null());
    }
}

/// Create default record
/// 
/// This allocates memory and must be destroyed by the caller
/// (see [`record_destroy`]).
#[no_mangle]
pub extern "C" fn record_default() -> *mut Record {
    let record = Record::default();
    Box::into_raw(Box::new(record))
}

/// Read record from file
/// 
/// This allocates memory and must be destroyed by the caller
/// (see [`record_destroy`]).
/// - A null pointer is returned if the filename is null, a file corresponding
/// to the filename does not exist, or the file cannot be read
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_read(filename: *const c_char) -> *mut Record {

    if filename.is_null() {
        update_error_code(ErrorCode::NullArgument);
        return std::ptr::null_mut()
    }

    let filename_string = match unsafe { CStr::from_ptr(filename) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid UTF encoding
            update_error_code(ErrorCode::InvalidUTF8String);
            return std::ptr::null_mut()
        }
    };

    let mut file = match File::open(filename_string) {
        Ok(f) => f,
        Err(err) => {
            map_io_error_to_error_code(err);
            return std::ptr::null_mut()
        }
    };

    let record = match Record::from_reader(&mut file) {
        Ok(r) => r,
        Err(err) => {
            map_record_error_to_error_code(err);
            return std::ptr::null_mut()
        }
    };

    Box::into_raw(Box::new(record))
}

/// Write record to file
///
/// This function will write to a filepath the from the contents
/// of the given Record.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_write(record: *mut Record, filename: *const c_char) -> c_int {
    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    if filename.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let filename_string = match unsafe { CStr::from_ptr(filename) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid UTF encoding
            return update_error_code(ErrorCode::InvalidUTF8String) as c_int
        }
    };

    let record_ref = unsafe { &*record };

    let mut file = match File::create(filename_string) {
        Ok(f) => f,
        Err(err) => {
            return map_io_error_to_error_code(err) as c_int
        }
    };

    if let Err(err) = record_ref.to_writer(&mut file) {
        return map_record_error_to_error_code(err) as c_int
    }

    ErrorCode::NoError as c_int
}

/// Get the record version
/// 
/// - If the [`Record`] pointer is null, null is returned.
/// - If the current version cannot be cast to [`std::ffi::CString`], null is returned.
/// - Returned version in null terminated
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_version(record: *mut Record) -> *const c_char {

    check_ptr_transform_to_cstring(record, |record_ref| { &record_ref.header.version[..] })
}


/// Set the record version
/// 
/// - If the [`Record`] pointer is null, the function does nothing and returns.
/// - If the version pointer is null, the function does nothing and returns.
/// - Input string should be UTF-8 encoded
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_set_version(record: *mut Record, version: *const c_char) -> c_int {

    set_str_value(record, version, |record_ref, string_version| {
        record_ref.header.version = string_version;
    }) as c_int
}

/// Get the record name
/// 
/// - If the [`Record`] pointer is null, null is returned.
/// - If the current name cannot be cast to [`std::ffi::CString`], null is returned.
/// - Returned name in null terminated
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_name(record: *mut Record) -> *const c_char {

    check_ptr_transform_to_cstring(record, |record_ref| { &record_ref.header.name[..] })
}

/// Set the record name
/// 
/// - If the [`Record`] pointer is null, the function does nothing and returns.
/// - If the name pointer is null, the function does nothing and returns.
/// - Input string should be UTF-8 encoded
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_set_name(record: *mut Record, name: *const c_char) -> c_int {

    set_str_value(record, name, |record_ref, string_name| {
        record_ref.header.name = string_name;
    }) as c_int
}

/// Get the number of comments
/// 
/// - If the [`Record`] pointer is null, zero is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_number_of_comments(record: *mut Record) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    unsafe { &*record }.header.comments.len() as c_int
}

/// Get an array of comments
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
/// - If index is out of bounds, a null pointer is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_comment(record: *mut Record, idx: size_t) -> *const c_char {

    check_ptr_index_transform_to_cstring(
        record, idx,
        |record_ref| { &record_ref.header.comments },
        |comments, idx| { &comments[idx] })
}

/// Append to an array of comments
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_append_comment(record: *mut Record, comment: *const c_char) -> c_int {

    set_str_value(record, comment, |record_ref, string_comment| {
        record_ref.header.comments.push(string_comment);
    }) as c_int
}

/// Get the number of devices
/// 
/// - If the [`Record`] pointer is null, zero is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_number_of_devices(record: *mut Record) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    unsafe { &*record }.header.devices.len() as c_int
}

/// Get the device name
/// 
/// - If the [`Record`] pointer is null, a null pointer is returned.
/// - If the index is out of bounds, a null pointer is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_device_name(record: *mut Record, idx: size_t) -> *const c_char {

    check_ptr_index_transform_to_cstring(
        record, idx,
        |record_ref| { &record_ref.header.devices },
        |devices, idx| { &devices[idx].name })
}

/// Append to an array of devices
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_append_device(record: *mut Record, name: *const c_char) -> c_int {

    set_str_value(record, name, |record_ref, device_name| {
        record_ref.header.devices.push(Device::new(&device_name));
    }) as c_int
}

/// Get the number of entries in a device
/// 
/// - If the [`Record`] pointer is null, zero.
/// - If the index is out of bounds, zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_device_number_of_entries(record: *mut Record, idx: size_t) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let record_ref = unsafe { &*record };
    if check_index_bounds(idx, record_ref.header.devices.len()) == false {
        return update_error_code(ErrorCode::IndexOutOfBounds) as c_int
    }

    record_ref.header.devices[idx].entries.len() as c_int
}

/// Get the entry from a device
/// 
/// - If the [`Record`] pointer is null, zero.
/// - If the index is out of bounds, zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_device_entry(record: *mut Record, device_idx: size_t, entry_idx: size_t) -> *const c_char {

    if record.is_null() {
        update_error_code(ErrorCode::NullArgument);
        return std::ptr::null_mut()
    }

    let record_ref = unsafe { &*record };
    if check_index_bounds(device_idx, record_ref.header.devices.len()) == false {
        update_error_code(ErrorCode::IndexOutOfBounds);
        return std::ptr::null_mut();
    }

    check_ptr_index_transform_to_cstring(
        record, entry_idx, 
        |record_ref| { &record_ref.header.devices[device_idx].entries },
        |entries, idx| { &entries[idx] })
}

/// Append to an array of entries for a given device
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_append_entry_to_device(
    record: *mut Record, device_idx: size_t, entry: *const c_char) -> c_int {

    if record.is_null() || entry.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let record_ref = unsafe { &mut *record };
    if check_index_bounds(device_idx, record_ref.header.devices.len()) == false {
        return update_error_code(ErrorCode::IndexOutOfBounds) as c_int
    }

    let entry_str = match unsafe { CStr::from_ptr(entry) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid utf8 encoding
            return ErrorCode::InvalidUTF8String as c_int
        }
    };

    record_ref.header.devices[device_idx].entries.push(entry_str);

    ErrorCode::NoError as c_int
}

/// Get independent variable name
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_name(record: *mut Record) -> *const c_char {

    check_ptr_transform_to_cstring(record, |record_ref| { &record_ref.header.independent_variable.name[..] })
}

/// Get independent variable format
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_format(record: *mut Record) -> *const c_char {

    check_ptr_transform_to_cstring(record, |record_ref| { &record_ref.header.independent_variable.format[..] })
}

/// Get independent variable length
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_length(record: *mut Record) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    unsafe { &*record }.header.independent_variable.data.len() as c_int
}

/// Get independent variable array
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_array(record: *mut Record) -> *const c_double {

    if record.is_null() {
        update_error_code(ErrorCode::NullArgument);
        return std::ptr::null_mut()
    }

    unsafe { &*record }.header.independent_variable.data.as_ptr()
}

/// Set independent variable array
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_set_independent_variable(
    record: *mut Record,
    name: *const c_char, format: *const c_char,
    vals: *const c_double, len: size_t) -> c_int {

    if record.is_null() || name.is_null() || format.is_null() || vals.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid utf8 encoding
            return ErrorCode::InvalidUTF8String as c_int
        }
    };

    let format_str = match unsafe { CStr::from_ptr(format) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid utf8 encoding
            return ErrorCode::InvalidUTF8String as c_int
        }
    };

    let vals_slice = unsafe { std::slice::from_raw_parts(vals, len) };
    let vals = vals_slice.iter().cloned().collect();

    let record_ref = unsafe { &mut *record };
    record_ref.header.independent_variable.name = name_str;
    record_ref.header.independent_variable.format = format_str;
    record_ref.header.independent_variable.data = vals;

    ErrorCode::NoError as c_int
}

/// Get number of data arrays
/// 
/// - If the [`Record`] pointer is null, return zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_number_of_data_arrays(record: *mut Record) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    unsafe { &*record }.data.len() as c_int
}

/// Get data array name
/// 
/// - If the [`Record`] pointer is null, return null pointer.
/// - If the index is out of bounds, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_name(record: *mut Record, idx: size_t) -> *const c_char {

    check_ptr_index_transform_to_cstring(
        record, idx,
        |record_ref| { &record_ref.data },
        |data, idx| { &data[idx].name })
}

/// Get data array format
/// 
/// - If the [`Record`] pointer is null, return zero.
/// - If the index is out of bounds, return zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_format(record: *mut Record, idx: size_t) -> *const c_char {

    check_ptr_index_transform_to_cstring(
        record, idx,
        |record_ref| { &record_ref.data },
        |data, idx| { &data[idx].format })
}

/// Get data array length
/// 
/// - If the [`Record`] pointer is null, return zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_length(record: *mut Record, idx: size_t) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let record_ref = unsafe { &*record };
    if check_index_bounds(idx, record_ref.data.len()) == false {
        return update_error_code(ErrorCode::IndexOutOfBounds) as c_int
    }

    record_ref.data[idx].samples.len() as c_int
}

/// Get data array
/// 
/// - If the [`Record`] pointer is null, nothing happens.
/// - If the index is out of bounds, nothing happens.
/// - Caller is responsible for allocation of the appropriate size and deallocation.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array(
    record: *mut Record,
    idx: size_t,
    real: *mut c_double, imag: *mut c_double) -> c_int {

    if record.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    if real.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }
    if imag.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let record_ref = unsafe { &*record };
    if check_index_bounds(idx, record_ref.data.len()) == false {
        return update_error_code(ErrorCode::IndexOutOfBounds) as c_int
    }

    // Fill array
    for (i, item) in record_ref.data[idx].samples.iter().enumerate() {
        unsafe {
            *real.offset(i as isize) = item.re;
            *imag.offset(i as isize) = item.im;
        }
    }

    ErrorCode::NoError as c_int
}

/// Append data array
/// 
/// - If the [`Record`] pointer is null, a corresponding error code is returned
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_append_data_array(
    record: *mut Record, name: *const c_char, format: *const c_char,
    reals: *const c_double, imags: *const c_double, len: size_t) -> c_int {

    if record.is_null() ||
        name.is_null() || format.is_null() ||
        reals.is_null() || imags.is_null() {
        return update_error_code(ErrorCode::NullArgument) as c_int
    }

    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid utf8 encoding
            return ErrorCode::InvalidUTF8String as c_int
        }
    };

    let format_str = match unsafe { CStr::from_ptr(format) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            // The only expected error is due to invalid utf8 encoding
            return ErrorCode::InvalidUTF8String as c_int
        }
    };

    let reals_slice = unsafe { std::slice::from_raw_parts(reals, len) };
    let imags_slice = unsafe { std::slice::from_raw_parts(imags, len) };
    let samples = reals_slice.iter()
        .zip(imags_slice.iter())
        .map(|(re, im)| Complex::<f64>::new(*re, *im))
        .collect();

    let record_ref = unsafe { &mut *record };
    record_ref.data.push(DataArray {
        name: name_str,
        format: format_str,
        samples: samples
    });

    ErrorCode::NoError as c_int
}

/// Create null pointer
#[cfg(test)]
fn null_setup() -> *mut Record {
    std::ptr::null_mut()
}

/// Create pointer from `record_default`
#[cfg(test)]
fn default_setup() -> *mut Record {
    record_default()
}

/// Release Record pointer
#[cfg(test)]
fn teardown(record_ptr: *mut Record) {
    record_destroy(record_ptr);
}

/// Test runner to handle creation and destruction of pointer
#[cfg(test)]
fn test_runner<S, T>(setup_fn: S, test_fn: T) -> () where
    S: Fn() -> *mut Record,
    T: FnOnce(*mut Record) -> () + std::panic::UnwindSafe
{
    // Setup
    let record_ptr: *mut Record = setup_fn();

    // Run test
    let result = std::panic::catch_unwind(|| {
        test_fn(record_ptr)
    });

    // Teardown
    teardown(record_ptr);

    // Assert
    assert!(result.is_ok())
}

#[cfg(test)]
mod test_runners {
    use super::*;

    mod null {
        use super::*;

        #[test]
        fn null_is_passed() {
            test_runner(null_setup, |record_ptr| {
                assert!(record_ptr.is_null());
            });
        }

        #[test]
        fn pass_passes() {
            test_runner(null_setup, |_record_ptr| {
                assert!(true);
            });
        }

        #[test]
        #[should_panic]
        fn fail_fails() {
            test_runner(null_setup, |_record_ptr| {
                assert!(false);
            });
        }
    }

    mod default {
        use super::*;

        #[test]
        fn not_null_is_passed() {
            test_runner(default_setup, |record_ptr| {
                assert!(!record_ptr.is_null());
            });
        }

        #[test]
        fn pass_passes() {
            test_runner(default_setup, |_record_ptr| {
                assert!(true);
            });
        }

        #[test]
        #[should_panic]
        fn fail_fails() {
            test_runner(default_setup, |_record_ptr| {
                assert!(false);
            });
        }
    }
}

#[cfg(test)]
mod interface {
    use super::*;

    mod record_get_version {
        use super::*;

        #[test]
        fn null() {
            test_runner(null_setup, |record_ptr| {
                let c_str = record_get_version(record_ptr);
                assert!(c_str.is_null());
            });
        }

        #[test]
        fn default() {
            test_runner(default_setup, unsafe { |record_ptr| {
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("A.01.00").unwrap()[..]);
            }});
        }
    }

    mod record_set_version {
        use super::*;

        #[test]
        fn null_record() {
            test_runner(null_setup, |record_ptr| {
                let version = CString::new("foo").unwrap().into_raw();
                record_set_version(record_ptr, version);
                let c_str = record_get_version(record_ptr);
                assert!(c_str.is_null());
            });
        }

        #[test]
        fn null_version() {
            test_runner(default_setup, unsafe { |record_ptr| {
                let version = std::ptr::null_mut();
                record_set_version(record_ptr, version);
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("A.01.00").unwrap()[..]);
            }});
        }

        #[test]
        fn set_version() {
            test_runner(default_setup, unsafe { |record_ptr| {
                let version = CString::new("foo").unwrap().into_raw();
                record_set_version(record_ptr, version);
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("foo").unwrap()[..]);
            }});
        }
    }

    mod record_get_name {
        use super::*;

        #[test]
        fn null() {
            test_runner(null_setup, |record_ptr| {
                let c_str = record_get_name(record_ptr);
                assert!(c_str.is_null());
            });
        }

        #[test]
        fn default() {
            test_runner(default_setup, unsafe { |record_ptr| {
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("").unwrap()[..]);
            }});
        }
    }

    mod record_set_name {
        use super::*;

        #[test]
        fn null_record() {
            test_runner(null_setup, |record_ptr| {
                let name = CString::new("foo").unwrap().into_raw();
                record_set_name(record_ptr, name);
                let c_str = record_get_name(record_ptr);
                assert!(c_str.is_null());
            });
        }

        #[test]
        fn null_version() {
            test_runner(default_setup, unsafe { |record_ptr| {
                let name = std::ptr::null_mut();
                record_set_name(record_ptr, name);
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("").unwrap()[..]);
            }});
        }

        #[test]
        fn set_version() {
            test_runner(default_setup, unsafe { |record_ptr| {
                let name = CString::new("foo").unwrap().into_raw();
                record_set_name(record_ptr, name);
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("foo").unwrap()[..]);
            }});
        }
    }

    mod record_get_number_of_comments {
        use super::*;

        #[test]
        fn null() {
            test_runner(null_setup, |record_ptr| {
                let count = record_get_number_of_comments(record_ptr);
                assert_eq!(count, ErrorCode::NullArgument as c_int);
            });
        }

        #[test]
        fn default() {
            test_runner(default_setup, |record_ptr| {
                let count = record_get_number_of_comments(record_ptr);
                assert_eq!(count, 0);
            });
        }
    }

    mod record_get_comment {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let comment = record_get_comment(record_ptr, 0_usize);
                assert!(comment.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let comment = record_get_comment(record_ptr, 0_usize);
                assert!(comment.is_null());
            });
        }
    }

    mod record_get_number_of_devices {
        use super::*;

        #[test]
        fn null() {
            test_runner(null_setup, |record_ptr| {
                let count = record_get_number_of_devices(record_ptr);
                assert_eq!(count, ErrorCode::NullArgument as c_int);
            });
        }

        #[test]
        fn default() {
            test_runner(default_setup, |record_ptr| {
                let count = record_get_number_of_devices(record_ptr);
                assert_eq!(count, 0);
            });
        }
    }

    mod record_get_device_name {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let comment = record_get_device_name(record_ptr, 0_usize);
                assert!(comment.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let comment = record_get_device_name(record_ptr, 0_usize);
                assert!(comment.is_null());
            });
        }
    }

    mod record_get_device_number_of_entries {
        use super::*;

        #[test]
        fn null() {
            test_runner(null_setup, |record_ptr| {
                let count = record_get_device_number_of_entries(record_ptr, 0_usize);
                assert_eq!(count, ErrorCode::NullArgument as c_int);
            });
        }

        #[test]
        fn default() {
            test_runner(default_setup, |record_ptr| {
                let count = record_get_device_number_of_entries(record_ptr, 0_usize);
                assert_eq!(count, ErrorCode::IndexOutOfBounds as c_int);
            });
        }
    }

    mod record_get_device_entry {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let comment = record_get_device_entry(record_ptr, 0_usize, 0_usize);
                assert!(comment.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let comment = record_get_device_entry(record_ptr, 0_usize, 0_usize);
                assert!(comment.is_null());
            });
        }
    }

    mod record_get_independent_variable_name {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let name = record_get_independent_variable_name(record_ptr);
                assert!(name.is_null());
            });
        }

        #[test]
        fn empty_returns_not_null() {
            test_runner(default_setup, |record_ptr| {
                let name = record_get_independent_variable_name(record_ptr);
                assert!(!name.is_null());
            });
        }
    }
    
    mod record_get_independent_variable_format {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let name = record_get_independent_variable_format(record_ptr);
                assert!(name.is_null());
            });
        }

        #[test]
        fn empty_returns_not_null() {
            test_runner(default_setup, |record_ptr| {
                let name = record_get_independent_variable_format(record_ptr);
                assert!(!name.is_null());
            });
        }
    }

    mod record_get_independent_variable_length {
        use super::*;

        #[test]
        fn null_returns_zero() {
            test_runner(null_setup, |record_ptr| {
                let length = record_get_independent_variable_length(record_ptr);
                assert_eq!(length, ErrorCode::NullArgument as c_int);
            });
        }

        #[test]
        fn empty_returns_zero() {
            test_runner(default_setup, |record_ptr| {
                let length = record_get_independent_variable_length(record_ptr);
                assert_eq!(length, 0);
            });
        }        
    }

    mod record_get_independent_variable_array {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let array = record_get_independent_variable_array(record_ptr);
                assert!(array.is_null());
            });
        }

        #[test]
        fn empty_returns_not_null() {
            test_runner(default_setup, |record_ptr| {
                let array = record_get_independent_variable_array(record_ptr);
                assert!(!array.is_null());
            });
        }
    }

    mod record_get_number_of_data_arrays {
        use super::*;

        #[test]
        fn null_returns_zero() {
            test_runner(null_setup, |record_ptr| {
                let number = record_get_number_of_data_arrays(record_ptr);
                assert_eq!(number, ErrorCode::NullArgument as c_int);
            });
        }

        #[test]
        fn empty_is_zero() {
            test_runner(default_setup, |record_ptr| {
                let number = record_get_number_of_data_arrays(record_ptr);
                assert_eq!(number, 0);
            });
        }
    }

    mod record_get_data_array_name{
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 0);
                assert!(name.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 0);
                assert!(name.is_null());
            });
        }
    }

    mod record_get_data_array_format {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 0);
                assert!(name.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 0);
                assert!(name.is_null());
            });
        }
    }

    mod record_get_data_array_length {
        use super::*;

        #[test]
        fn null_returns_zero() {
            test_runner(null_setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                assert_eq!(number, ErrorCode::NullArgument as c_int);
            });
        }

        #[test]
        fn empty_is_zero() {
            test_runner(default_setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                println!("arr len: {:}", number);
                assert_eq!(number, ErrorCode::IndexOutOfBounds as c_int);
            });
        }
    }
}

#[cfg(test)]
mod read {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn null_filename() {
        let record_ptr: *mut Record = record_read(std::ptr::null_mut());

        let result = std::panic::catch_unwind(|| {
            assert!(record_ptr.is_null());
        });
        record_destroy(record_ptr);
        assert!(result.is_ok())
    }

    #[test]
    fn non_existant_file() {
        let record_ptr: *mut Record = record_read(CString::new("this is a file that does not exist").unwrap().into_raw());

        let result = std::panic::catch_unwind(|| {
            assert!(record_ptr.is_null());
        });
        record_destroy(record_ptr);
        assert!(result.is_ok())
    }

    #[cfg(test)]
    fn data_directory() -> PathBuf {
        let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_buf.push("tests");
        path_buf.push("regression_files");
        path_buf
    }

    mod display_memory_record {
        use super::*;

        fn setup() -> *mut Record {
            // PathBuf
            let mut path_buf = data_directory();
            path_buf.push("display_memory.cti");

            // Read
            record_read(CString::new(path_buf.into_os_string().into_string().unwrap()).unwrap().into_raw())
        }

        #[test]
        fn name() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("MEMORY").unwrap()[..]);
            }});
        }

        #[test]
        fn version() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("A.01.00").unwrap()[..]);
            }});
        }

        #[test]
        fn can_read_file() {
            test_runner(setup, |record_ptr| {
                assert!(!record_ptr.is_null());
            });
        }

        #[test]
        fn record_get_number_of_comments_is_zero() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_comments(record_ptr), 0);
            });
        }

        #[test]
        fn record_get_number_of_devices_is_one() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_devices(record_ptr), 1);
            });
        }

        #[test]
        fn record_get_device_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_name(record_ptr, 0_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("NA").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_device_number_of_entries_is_two() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_device_number_of_entries(record_ptr, 0_usize), 2);
            });
        }

        #[test]
        fn record_get_device_entry_zero() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_entry(record_ptr, 0_usize, 0_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("VERSION HP8510B.05.00").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_device_entry_one() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_entry(record_ptr, 0_usize, 1_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("REGISTER 1").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_name(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("FREQ").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_format_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_format(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("MAG").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_length_is_zero() {
            test_runner(setup, |record_ptr| {
                let length = record_get_independent_variable_length(record_ptr);
                assert_eq!(length, 0);
            });
        }

        #[test]
        fn record_get_independent_variable_array_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_independent_variable_array(record_ptr);
                assert!(!array.is_null());
            });   
        }

        #[test]
        fn record_get_number_of_data_arrays_is_one() {
            test_runner(setup, |record_ptr| {
                let number = record_get_number_of_data_arrays(record_ptr);
                assert_eq!(number, 1);
            });
        }

        #[test]
        fn record_get_data_array_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("S").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_format_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("RI").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_length_is_five() {
            test_runner(setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                assert_eq!(number, 5);
            });
        }
    }

    mod data_record {
        use super::*;

        fn setup() -> *mut Record {
            // PathBuf
            let mut path_buf = data_directory();
            path_buf.push("data_file.cti");

            // Read
            record_read(CString::new(path_buf.into_os_string().into_string().unwrap()).unwrap().into_raw())
        }

        #[test]
        fn can_read_file() {
            test_runner(setup, |record_ptr| {
                assert!(!record_ptr.is_null());
            });
        }

        #[test]
        fn name() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("DATA").unwrap()[..]);
            }});
        }

        #[test]
        fn version() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("A.01.00").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_number_of_comments_is_zero() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_comments(record_ptr), 0);
            });
        }

        #[test]
        fn record_get_number_of_devices_is_one() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_devices(record_ptr), 1);
            });
        }

        #[test]
        fn record_get_device_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_name(record_ptr, 0_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("NA").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_device_number_of_entries_is_two() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_device_number_of_entries(record_ptr, 0_usize), 2);
            });
        }

        #[test]
        fn record_get_device_entry_zero() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_entry(record_ptr, 0_usize, 0_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("VERSION HP8510B.05.00").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_device_entry_one() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_entry(record_ptr, 0_usize, 1_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("REGISTER 1").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_name(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("FREQ").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_format_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_format(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("MAG").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_length_is_ten() {
            test_runner(setup, |record_ptr| {
                let length = record_get_independent_variable_length(record_ptr);
                assert_eq!(length, 10);
            });
        }

        #[test]
        fn record_get_independent_variable_array_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_independent_variable_array(record_ptr);
                assert!(!array.is_null());
            });   
        }
    
        #[test]
        fn record_get_number_of_data_arrays_is_one() {
            test_runner(setup, |record_ptr| {
                let number = record_get_number_of_data_arrays(record_ptr);
                assert_eq!(number, 1);
            });    
        }

        #[test]
        fn record_get_data_array_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("S[1,1]").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_format_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("RI").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_length_is_ten() {
            test_runner(setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                assert_eq!(number, 10);
            });
        }
    }

    mod list_cal_set_record {
        use super::*;

        fn setup() -> *mut Record {
            // PathBuf
            let mut path_buf = data_directory();
            path_buf.push("list_cal_set.cti");

            // Read
            record_read(CString::new(path_buf.into_os_string().into_string().unwrap()).unwrap().into_raw())
        }

        #[test]
        fn can_read_file() {
            test_runner(setup, |record_ptr| {
                assert!(!record_ptr.is_null());
            });
        }

        #[test]
        fn name() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("CAL_SET").unwrap()[..]);
            }});
        }

        #[test]
        fn version() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("A.01.00").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_number_of_comments_is_zero() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_comments(record_ptr), 0);
            });
        }

        #[test]
        fn record_get_number_of_devices_is_one() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_devices(record_ptr), 1);
            });
        }

        #[test]
        fn record_get_device_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let comment = record_get_device_name(record_ptr, 0_usize);
                assert!(!comment.is_null());
                assert_eq!(CStr::from_ptr(comment), &CString::new("NA").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_device_number_of_entries_is_seventeen() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_device_number_of_entries(record_ptr, 0_usize), 17);
            });
        }

        #[test]
        fn record_get_device_entries() {
            test_runner(setup, unsafe { |record_ptr| {
                let expected = vec![
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
                ];

                for (i, item) in expected.iter().enumerate() {
                    let comment = record_get_device_entry(record_ptr, 0_usize, i as usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new(*item).unwrap()[..]);
                }
            }});
        }

        #[test]
        fn record_get_independent_variable_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_name(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("FREQ").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_format_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_format(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("MAG").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_length_is_four() {
            test_runner(setup, |record_ptr| {
                let length = record_get_independent_variable_length(record_ptr);
                assert_eq!(length, 4);
            });
        }

        #[test]
        fn record_get_independent_variable_array_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_independent_variable_array(record_ptr);
                assert!(!array.is_null());
            });   
        }

        #[test]
        fn record_get_number_of_data_arrays_is_three() {
            test_runner(setup, |record_ptr| {
                let number = record_get_number_of_data_arrays(record_ptr);
                assert_eq!(number, 3);
            });    
        }

        #[test]
        fn record_get_data_array_name_zero_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("E[1]").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_length_zero_is_four() {
            test_runner(setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                assert_eq!(number, 4);
            });
        }

        #[test]
        fn record_get_data_array_length_one_is_four() {
            test_runner(setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 1);
                assert_eq!(number, 4);
            });
        }

        #[test]
        fn record_get_data_array_length_two_is_four() {
            test_runner(setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 2);
                assert_eq!(number, 4);
            });
        }

        #[test]
        fn record_get_data_array_name_one_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 1);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("E[2]").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_name_two_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 2);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("E[3]").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_format_zero_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("RI").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_format_one_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 1);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("RI").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_format_two_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 2);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("RI").unwrap()[..]);
            }});
        }
    }

    mod wvi_record {
        use super::*;

        fn setup() -> *mut Record {
            // PathBuf
            let mut path_buf = data_directory();
            path_buf.push("wvi_file.cti");

            // Read
            record_read(CString::new(path_buf.into_os_string().into_string().unwrap()).unwrap().into_raw())
        }

        #[test]
        fn can_read_file() {
            test_runner(setup, |record_ptr| {
                assert!(!record_ptr.is_null());
            });
        }

        #[test]
        fn name() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_name(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("Antonly001").unwrap()[..]);
            }});
        }

        #[test]
        fn version() {
            test_runner(setup, unsafe { |record_ptr| {
                let c_str = record_get_version(record_ptr);
                assert!(!c_str.is_null());
                assert_eq!(CStr::from_ptr(c_str), &CString::new("A.01.01").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_number_of_comments_is_six() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_comments(record_ptr), 6);
            });
        }

        mod comments {
            use super::*;

            #[test]
            fn comment_zero() {
                test_runner(setup, unsafe { |record_ptr| {
                    let comment = record_get_comment(record_ptr, 0_usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new("SOURCE: 10095059066467").unwrap()[..]);
                }});
            }

            #[test]
            fn comment_one() {
                test_runner(setup, unsafe { |record_ptr| {
                    let comment = record_get_comment(record_ptr, 1_usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new("DATE: Fri, Jan 18, 2019, 14:14:44").unwrap()[..]);
                }});
            }

            #[test]
            fn comment_two() {
                test_runner(setup, unsafe { |record_ptr| {
                    let comment = record_get_comment(record_ptr, 2_usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new("ANTPOS_TX: 28.4E-3 0E+0 -16E-3 90 270 0").unwrap()[..]);
                }});
            }

            #[test]
            fn comment_three() {
                test_runner(setup, unsafe { |record_ptr| {
                    let comment = record_get_comment(record_ptr, 3_usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new("ANTPOS_RX: 28.4E-3 0E+0 -16E-3 90 270 0").unwrap()[..]);
                }});
            }

            #[test]
            fn comment_four() {
                test_runner(setup, unsafe { |record_ptr| {
                    let comment = record_get_comment(record_ptr, 4_usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new("ANT_TX: NAH_003").unwrap()[..]);
                }});
            }

            #[test]
            fn comment_five() {
                test_runner(setup, unsafe { |record_ptr| {
                    let comment = record_get_comment(record_ptr, 5_usize);
                    assert!(!comment.is_null());
                    assert_eq!(CStr::from_ptr(comment), &CString::new("ANT_RX: NAH_003").unwrap()[..]);
                }});
            }
        }

        #[test]
        fn record_get_number_of_devices_is_zero() {
            test_runner(setup, |record_ptr| {
                assert_eq!(record_get_number_of_devices(record_ptr), 0);
            });
        }

        #[test]
        fn record_get_independent_variable_name_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_name(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("Freq").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_format_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_independent_variable_format(record_ptr);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("MAG").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_independent_variable_length_is_two() {
            test_runner(setup, |record_ptr| {
                let length = record_get_independent_variable_length(record_ptr);
                assert_eq!(length, 2);
            });
        }

        #[test]
        fn record_get_independent_variable_array_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_independent_variable_array(record_ptr);
                assert!(!array.is_null());
            });   
        }

        #[test]
        fn record_get_number_of_data_arrays_is_one() {
            test_runner(setup, |record_ptr| {
                let number = record_get_number_of_data_arrays(record_ptr);
                assert_eq!(number, 1);
            });    
        }

        #[test]
        fn record_get_data_array_name_zero_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_name(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("S11").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_format_zero_is_correct() {
            test_runner(setup, unsafe { |record_ptr| {
                let name = record_get_data_array_format(record_ptr, 0);
                assert!(!name.is_null());
                assert_eq!(CStr::from_ptr(name), &CString::new("RI").unwrap()[..]);
            }});
        }

        #[test]
        fn record_get_data_array_length_is_two() {
            test_runner(setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                assert_eq!(number, 2);
            });
        }
    }
}
