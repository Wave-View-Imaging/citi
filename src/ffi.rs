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
use libc::c_char;

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
}
