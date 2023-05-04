use std::ops::Deref;
use std::ptr::drop_in_place;
use std::str;

use crate::vec::Vec;

#[repr(C)]
pub struct String(Vec<u8>);

impl From<std::string::String> for String {
    #[inline]
    fn from(value: std::string::String) -> Self {
        String(Vec::from(value.into_bytes()))
    }
}

impl Into<std::string::String> for String {
    #[inline]
    fn into(self) -> std::string::String {
        unsafe { std::string::String::from_utf8_unchecked(self.0.into()) }
    }
}

impl Deref for String {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

/// cbindgen:ignore
type Void = String;

#[no_mangle]
pub unsafe extern "C" fn crust_string_len(string: *const Void) -> usize {
    (&*string).0.len()
}

#[no_mangle]
pub unsafe extern "C" fn crust_string_at(string: *const Void, i: usize) -> *const u8 {
    &(&*string).0[i]
}

#[no_mangle]
pub unsafe extern "C" fn crust_string_data(string: *const Void) -> *const u8 {
    (*string).0.as_ref().as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn crust_string_free(string: *mut Void) {
    drop_in_place(string)
}
