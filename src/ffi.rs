//! Foreign Function Interface
//! 
//! This FFI was designed to work as follows:
//! - Rust creates and destorys a pointer to a [Record]
//! - The interfacing language holds the pointer and is
//! responsible for creation, destruction, and avoiding
//! memory leaks.
//! - Rust methods take the pointer and modify the pointer
//! or return a value based on the interface.
use crate::Record;

use std::ffi::{CString, CStr};
use libc::{c_char, size_t, c_double};
use std::fs::File;

/// Free a pointer to `Record`
/// 
/// This can be called on `null`. After being freed, the pointer
/// is left dangling, still pointing to the freed memory.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_destroy(record: *mut Record) {
    if !record.is_null() {
        unsafe {
            drop(Box::from_raw(record));
        }
    }
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
    // Check null filename
    if filename.is_null() {
        return std::ptr::null_mut();
    }

    // Filename string
    let filename_string = unsafe { match CStr::from_ptr(filename).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return std::ptr::null_mut(),
    }};

    // Setup file
    let mut file = match File::open(filename_string) {
        Ok(f) => f,
        Err(_) => return std::ptr::null_mut(),
    };

    // Read and return
    let record = match Record::from_reader(&mut file) {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };
    Box::into_raw(Box::new(record))
}

/// Get the record version
/// 
/// - If the [`Record`] pointer is null, null is returned.
/// - If the current version cannot be cast to [`std::ffi::CString`], null is returned.
/// - Returned version in null terminated
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_version(record: *mut Record) -> *const c_char {
    // Check null
    if record.is_null() {
        return std::ptr::null_mut();
    }

    // Convert to C string. Going through CString adds null terminator.
    let c_str = unsafe {
        match CString::new(&(*record).header.version[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    c_str.into_raw()
}

/// Set the record version
/// 
/// - If the [`Record`] pointer is null, the function does nothing and returns.
/// - If the version pointer is null, the function does nothing and returns.
/// - Input string should be UTF-8 encoded
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_set_version(record: *mut Record, version: *const c_char) {
    // Check null record
    if record.is_null() {
        return;
    }

    // Check null version
    if version.is_null() {
        return;
    }

    // Convert to String and set
    unsafe {
        let string_version = match CStr::from_ptr(version).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        };
        (*record).header.version = string_version;
    }
}

/// Get the record name
/// 
/// - If the [`Record`] pointer is null, null is returned.
/// - If the current name cannot be cast to [`std::ffi::CString`], null is returned.
/// - Returned name in null terminated
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_name(record: *mut Record) -> *const c_char {
    // Check null
    if record.is_null() {
        return std::ptr::null_mut();
    }

    // Convert to C string. Going through CString adds null terminator.
    let c_str = unsafe {
        match CString::new(&(*record).header.name[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    c_str.into_raw()
}

/// Set the record name
/// 
/// - If the [`Record`] pointer is null, the function does nothing and returns.
/// - If the name pointer is null, the function does nothing and returns.
/// - Input string should be UTF-8 encoded
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_set_name(record: *mut Record, name: *const c_char) {
    // Check null record
    if record.is_null() {
        return;
    }

    // Check null name
    if name.is_null() {
        return;
    }

    // Convert to String and set
    unsafe {
        let string_name = match CStr::from_ptr(name).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        };
        (*record).header.name = string_name;
    }
}

/// Get the number of comments
/// 
/// - If the [`Record`] pointer is null, zero is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_number_of_comments(record: *mut Record) -> size_t {
    // Check null record
    if record.is_null() {
        return 0_usize;
    }

    // Get length
    unsafe {
        (*record).header.comments.len()
    }
}

/// Get an array of comments
/// 
/// - If the [`Record`] pointer is null, a null pointer is returned.
/// - If index is out of bounds, a null pointer is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_comment(record: *mut Record, idx: size_t) ->*const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check size
        if idx >= (*record).header.comments.len() {
            return std::ptr::null_mut();
        }

        // Get value
        let c_str = match CString::new(&(*record).header.comments[idx][..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get the number of devices
/// 
/// - If the [`Record`] pointer is null, zero is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_number_of_devices(record: *mut Record) -> size_t {
    // Check null record
    if record.is_null() {
        return 0_usize;
    }

    // Get length
    unsafe {
        (*record).header.devices.len()
    }
}

/// Get the device name
/// 
/// - If the [`Record`] pointer is null, a null pointer is returned.
/// - If the index is out of bounds, a null pointer is returned.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_device_name(record: *mut Record, idx: size_t) -> *const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check size
        if idx >= (*record).header.devices.len() {
            return std::ptr::null_mut();
        }

        // Get value
        let c_str = match CString::new(&(*record).header.devices[idx].name[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get the number of entries in a device
/// 
/// - If the [`Record`] pointer is null, zero.
/// - If the index is out of bounds, zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_device_number_of_entries(record: *mut Record, idx: size_t) -> size_t {
    // Check null record
    if record.is_null() {
        return 0_usize;
    }

    unsafe {
        // Check size
        if idx >= (*record).header.devices.len() {
            return 0_usize;
        }

        // Get length
        (*record).header.devices[idx].entries.len()
    }
}

/// Get the entry from a device
/// 
/// - If the [`Record`] pointer is null, zero.
/// - If the index is out of bounds, zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_device_entry(record: *mut Record, device_idx: size_t, entry_idx: size_t) -> *const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check device index
        if device_idx >= (*record).header.devices.len() {
            return std::ptr::null_mut();
        }

        // Check entry index
        if entry_idx >= (*record).header.devices[device_idx].entries.len() {
            return std::ptr::null_mut();
        }

        // Get value
        let c_str = match CString::new(&(*record).header.devices[device_idx].entries[entry_idx][..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get independent variable name
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_name(record: *mut Record) -> *const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Get value
        let c_str = match CString::new(&(*record).header.independent_variable.name[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get independent variable format
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_format(record: *mut Record) -> *const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Get value
        let c_str = match CString::new(&(*record).header.independent_variable.format[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get independent variable length
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_length(record: *mut Record) -> size_t {
    // Check null record
    if record.is_null() {
        return 0_usize;
    }

    unsafe {
        // Get length
        (*record).header.independent_variable.data.len()
    }
}

/// Get independent variable array
/// 
/// - If the [`Record`] pointer is null, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_independent_variable_array(record: *mut Record) -> *const c_double {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        (*record).header.independent_variable.data.as_mut_ptr()
    }
}

/// Get number of data arrays
/// 
/// - If the [`Record`] pointer is null, return zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_number_of_data_arrays(record: *mut Record) -> size_t {
    // Check null record
    if record.is_null() {
        return 0_usize;
    }

    unsafe {
        // Get length
        (*record).data.len()
    }
}

/// Get data array name
/// 
/// - If the [`Record`] pointer is null, return null pointer.
/// - If the index is out of bounds, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_name(record: *mut Record, idx: size_t) -> *const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check index
        if idx >= (*record).data.len() {
            return std::ptr::null_mut();
        }

        // Get value
        let c_str = match CString::new(&(*record).data[idx].name[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get data array format
/// 
/// - If the [`Record`] pointer is null, return zero.
/// - If the index is out of bounds, return zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_format(record: *mut Record, idx: size_t) -> *const c_char {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check index
        if idx >= (*record).data.len() {
            return std::ptr::null_mut();
        }

        // Get value
        let c_str = match CString::new(&(*record).data[idx].format[..]) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        c_str.into_raw()
    }
}

/// Get data array length
/// 
/// - If the [`Record`] pointer is null, return zero.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_length(record: *mut Record, idx: size_t) -> size_t {
    // Check null record
    if record.is_null() {
        return 0_usize;
    }

    unsafe {
        // Check index
        if idx >= (*record).data.len() {
            return 0_usize;
        }

        (*record).data[idx].samples.len()
    }
}

/// Get real array from data array
/// 
/// - If the [`Record`] pointer is null, return null pointer.
/// - If the index is out of bounds, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_real_component(record: *mut Record, idx: size_t) -> *mut c_double {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check index
        if idx >= (*record).data.len() {
            return std::ptr::null_mut();
        }

        let real_ptr = (*record).data[idx].samples.clone().into_iter().map(|x| x.re).collect::<Vec<f64>>().as_mut_ptr();
        std::mem::forget(real_ptr);
        real_ptr
    }
}

/// Get imaginary array from data array
/// 
/// - If the [`Record`] pointer is null, return null pointer.
/// - If the index is out of bounds, return null pointer.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn record_get_data_array_imag_component(record: *mut Record, idx: size_t) -> *mut c_double {
    // Check null record
    if record.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        // Check index
        if idx >= (*record).data.len() {
            return std::ptr::null_mut();
        }

        let imag_ptr = (*record).data[idx].samples.clone().into_iter().map(|x| x.im).collect::<Vec<f64>>().as_mut_ptr();
        std::mem::forget(imag_ptr);
        imag_ptr
    }
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
                assert_eq!(count, 0);
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
                assert_eq!(count, 0);
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
                assert_eq!(count, 0);
            });
        }

        #[test]
        fn default() {
            test_runner(default_setup, |record_ptr| {
                let count = record_get_device_number_of_entries(record_ptr, 0_usize);
                assert_eq!(count, 0);
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
                assert_eq!(length, 0);
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
                assert_eq!(number, 0);
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
                assert_eq!(number, 0);
            });
        }

        #[test]
        fn empty_is_zero() {
            test_runner(default_setup, |record_ptr| {
                let number = record_get_data_array_length(record_ptr, 0);
                assert_eq!(number, 0);
            });
        }
    }

    mod record_get_data_array_real_component {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let name = record_get_data_array_real_component(record_ptr, 0);
                assert!(name.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let name = record_get_data_array_real_component(record_ptr, 0);
                assert!(name.is_null());
            });
        }
    }

    mod record_get_data_array_imag_component {
        use super::*;

        #[test]
        fn null_returns_null() {
            test_runner(null_setup, |record_ptr| {
                let name = record_get_data_array_imag_component(record_ptr, 0);
                assert!(name.is_null());
            });
        }

        #[test]
        fn empty_returns_null() {
            test_runner(default_setup, |record_ptr| {
                let name = record_get_data_array_imag_component(record_ptr, 0);
                assert!(name.is_null());
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

        #[test]
        fn record_get_data_array_real_component_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_real_component(record_ptr, 0);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_imag_component_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_imag_component(record_ptr, 0);
                assert!(!array.is_null());
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

        #[test]
        fn record_get_data_array_real_component_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_real_component(record_ptr, 0);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_imag_component_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_imag_component(record_ptr, 0);
                assert!(!array.is_null());
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
                assert_eq!(record_get_number_of_comments(record_ptr), 0_usize);
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

        #[test]
        fn record_get_data_array_real_component_zero_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_real_component(record_ptr, 0);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_real_component_one_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_real_component(record_ptr, 1);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_real_component_two_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_real_component(record_ptr, 2);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_imag_component_zero_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_imag_component(record_ptr, 0);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_imag_component_one_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_imag_component(record_ptr, 1);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_imag_component_two_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_imag_component(record_ptr, 2);
                assert!(!array.is_null());
            });
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

        #[test]
        fn record_get_data_array_real_component_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_real_component(record_ptr, 0);
                assert!(!array.is_null());
            });
        }

        #[test]
        fn record_get_data_array_imag_component_is_not_null() {
            test_runner(setup, |record_ptr| {
                let array = record_get_data_array_imag_component(record_ptr, 0);
                assert!(!array.is_null());
            });
        }
    }
}
