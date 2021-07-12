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

use std::ffi::CString;
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

