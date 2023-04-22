use std::ops::Deref;
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
