/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! A replacement for `Box<[T]>` that cbindgen can understand.

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::{fmt, iter, mem, slice};

/// A struct that basically replaces a `Box<[T]>`, but which cbindgen can
/// understand.
///
/// We could rely on the struct layout of `Box<[T]>` per:
///
///   https://github.com/rust-lang/unsafe-code-guidelines/blob/master/reference/src/layout/pointers.md
///
/// But handling fat pointers with cbindgen both in structs and argument
/// positions more generally is a bit tricky.
///
/// cbindgen:derive-eq=false
/// cbindgen:derive-neq=false
#[repr(C)]
pub struct OwnedSlice<T: Sized> {
    ptr: NonNull<T>,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T: Sized> Default for OwnedSlice<T> {
    #[inline]
    fn default() -> Self {
        Self {
            len: 0,
            ptr: NonNull::dangling(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Sized> Drop for OwnedSlice<T> {
    #[inline]
    fn drop(&mut self) {
        if self.len != 0 {
            let _ = mem::replace(self, Self::default()).into_vec();
        }
    }
}

unsafe impl<T: Sized + Send> Send for OwnedSlice<T> {}
unsafe impl<T: Sized + Sync> Sync for OwnedSlice<T> {}

impl<T: Clone> Clone for OwnedSlice<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_slice(&**self)
    }
}

impl<T: fmt::Debug> fmt::Debug for OwnedSlice<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(formatter)
    }
}

impl<T: PartialEq> PartialEq for OwnedSlice<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<T: Eq> Eq for OwnedSlice<T> {}

impl<T: Sized> OwnedSlice<T> {
    /// Convert the OwnedSlice into a boxed slice.
    #[inline]
    pub fn into_box(self) -> Box<[T]> {
        self.into_vec().into_boxed_slice()
    }

    /// Convert the OwnedSlice into a Vec.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        let ret = unsafe { Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.len) };
        mem::forget(self);
        ret
    }

    /// Iterate over all the elements in the slice taking ownership of them.
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = T> + ExactSizeIterator {
        self.into_vec().into_iter()
    }

    /// Convert the regular slice into an owned slice.
    #[inline]
    pub fn from_slice(s: &[T]) -> Self
    where
        T: Clone,
    {
        Self::from(s.to_vec())
    }
}

impl<T> Deref for OwnedSlice<T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for OwnedSlice<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> From<Box<[T]>> for OwnedSlice<T> {
    #[inline]
    fn from(mut b: Box<[T]>) -> Self {
        let len = b.len();
        let ptr = unsafe { NonNull::new_unchecked(b.as_mut_ptr()) };
        mem::forget(b);
        Self {
            len,
            ptr,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Vec<T>> for OwnedSlice<T> {
    #[inline]
    fn from(b: Vec<T>) -> Self {
        Self::from(b.into_boxed_slice())
    }
}

impl<T> iter::FromIterator<T> for OwnedSlice<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Vec::from_iter(iter).into()
    }
}

impl<T: Serialize> Serialize for OwnedSlice<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for OwnedSlice<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r = Box::<[T]>::deserialize(deserializer)?;
        Ok(r.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_slice() {
        let record = r#"
        [1,2,3,4,5]
        "#;

        let result: OwnedSlice<i32> = serde_json::from_str(record).unwrap();
        assert_eq!(3, result[2]);
        let result_two: OwnedSlice<i32> = serde_json::from_str(
            &serde_json::to_string(&result).expect("Failed to serialize result"),
        )
        .expect("Failed to serialize json");
        assert_eq!(result, result_two);
    }
}
