// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The base implementation of CString and CStr. The CString type
//! is wrapped again in std only so that memchr can be implemented
//! per-platform. Tests are in std.

#![crate_name = "c_str"]
#![unstable(feature = "pal", reason = "unstable", issue = "0")]
#![crate_type = "rlib"]
#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://doc.rust-lang.org/favicon.ico",
       html_root_url = "https://doc.rust-lang.org/nightly/",
       html_playground_url = "https://play.rust-lang.org/",
       issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
       test(no_crate_inject, attr(deny(warnings))),
       test(attr(allow(dead_code, deprecated, unused_variables, unused_mut))))]

#![no_std]

#![feature(alloc)]
#![feature(collections)]
#![feature(libc)]
#![feature(pal)]
#![feature(question_mark)]
#![feature(staged_api)]

extern crate alloc;
extern crate collections;
extern crate libc;
extern crate pal_common;

use pal_common::ascii;
use collections::borrow::{Cow, Borrow};
use core::cmp::Ordering;
use pal_common::error::Error;
use core::fmt::{self, Write};
use core::mem;
use pal_common::memchr;
use core::ops;
use libc::c_char;
use core::ptr;
use core::slice;
use core::str::{self, Utf8Error};
use collections::vec::Vec;
use collections::string::String;
use alloc::boxed::Box;
use collections::borrow::ToOwned;

pub fn _new_nul_error(i: usize, b: Vec<u8>) -> NulError { NulError(i, b) }
pub fn _new_from_bytes_with_nul_error() -> FromBytesWithNulError {
    FromBytesWithNulError { _a: () }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone)]
pub struct CString {
    // Invariant 1: the slice ends with a zero byte and has a length of at least one.
    // Invariant 2: the slice contains only one zero byte.
    // Improper usage of unsafe function can break Invariant 2, but not Invariant 1.
    inner: Box<[u8]>,
}

#[derive(Hash)]
pub struct CStr {
    // FIXME: this should not be represented with a DST slice but rather with
    //        just a raw `c_char` along with some form of marker to make
    //        this an unsized type. Essentially `sizeof(&CStr)` should be the
    //        same as `sizeof(&c_char)` but `CStr` should be an unsized type.
    inner: [c_char]
}

/// An error returned from `CString::new` to indicate that a nul byte was found
/// in the vector provided.
#[derive(Clone, PartialEq, Eq, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct NulError(usize, Vec<u8>);

/// An error returned from `CStr::from_bytes_with_nul` to indicate that a nul
/// byte was found too early in the slice provided or one wasn't found at all.
#[derive(Clone, PartialEq, Eq, Debug)]
#[stable(feature = "cstr_from_bytes", since = "1.10.0")]
pub struct FromBytesWithNulError { _a: () }

/// An error returned from `CString::into_string` to indicate that a UTF-8 error
/// was encountered during the conversion.
#[derive(Clone, PartialEq, Eq, Debug)]
#[stable(feature = "cstring_into", since = "1.7.0")]
pub struct IntoStringError {
    inner: CString,
    error: Utf8Error,
}

impl CString {
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<CString, NulError> {
        Self::_new(t.into())
    }

    fn _new(bytes: Vec<u8>) -> Result<CString, NulError> {
        match memchr::fallback::memchr(0, &bytes) {
            Some(i) => Err(NulError(i, bytes)),
            None => Ok(unsafe { CString::from_vec_unchecked(bytes) }),
        }
    }

    pub unsafe fn from_vec_unchecked(mut v: Vec<u8>) -> CString {
        v.reserve_exact(1);
        v.push(0);
        CString { inner: v.into_boxed_slice() }
    }

    pub unsafe fn from_raw(ptr: *mut c_char) -> CString {
        let len = libc::strlen(ptr) + 1; // Including the NUL byte
        let slice = slice::from_raw_parts(ptr, len as usize);
        CString { inner: mem::transmute(slice) }
    }

    pub fn into_raw(self) -> *mut c_char {
        Box::into_raw(self.into_inner()) as *mut c_char
    }

    pub fn into_string(self) -> Result<String, IntoStringError> {
        String::from_utf8(self.into_bytes())
            .map_err(|e| IntoStringError {
                error: e.utf8_error(),
                inner: unsafe { CString::from_vec_unchecked(e.into_bytes()) },
            })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut vec = self.into_inner().into_vec();
        let _nul = vec.pop();
        debug_assert_eq!(_nul, Some(0u8));
        vec
    }

    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.into_inner().into_vec()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner[..self.inner.len() - 1]
    }

    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.inner
    }

    // Bypass "move out of struct which implements `Drop` trait" restriction.
    fn into_inner(self) -> Box<[u8]> {
        unsafe {
            let result = ptr::read(&self.inner);
            mem::forget(self);
            result
        }
    }
}

// Turns this `CString` into an empty string to prevent
// memory unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl Drop for CString {
    #[inline]
    fn drop(&mut self) {
        unsafe { *self.inner.get_unchecked_mut(0) = 0; }
    }
}

impl ops::Deref for CString {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        unsafe { mem::transmute(self.as_bytes_with_nul()) }
    }
}

impl fmt::Debug for CString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl From<CString> for Vec<u8> {
    fn from(s: CString) -> Vec<u8> {
        s.into_bytes()
    }
}

impl fmt::Debug for CStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        for byte in self.to_bytes().iter().flat_map(|&b| ascii::escape_default(b)) {
            f.write_char(byte as char)?;
        }
        write!(f, "\"")
    }
}

impl<'a> Default for &'a CStr {
    fn default() -> &'a CStr {
        static SLICE: &'static [c_char] = &[0];
        unsafe { CStr::from_ptr(SLICE.as_ptr()) }
    }
}

#[stable(feature = "cstr_default", since = "1.10.0")]
impl Default for CString {
    /// Creates an empty `CString`.
    fn default() -> CString {
        let a: &CStr = Default::default();
        a.to_owned()
    }
}

#[stable(feature = "cstr_borrow", since = "1.3.0")]
impl Borrow<CStr> for CString {
    fn borrow(&self) -> &CStr { self }
}

impl NulError {
    /// Returns the position of the nul byte in the slice that was provided to
    /// `CString::new`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::CString;
    ///
    /// let nul_error = CString::new("foo\0bar").unwrap_err();
    /// assert_eq!(nul_error.nul_position(), 3);
    ///
    /// let nul_error = CString::new("foo bar\0").unwrap_err();
    /// assert_eq!(nul_error.nul_position(), 7);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn nul_position(&self) -> usize { self.0 }

    /// Consumes this error, returning the underlying vector of bytes which
    /// generated the error in the first place.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::CString;
    ///
    /// let nul_error = CString::new("foo\0bar").unwrap_err();
    /// assert_eq!(nul_error.into_vec(), b"foo\0bar");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_vec(self) -> Vec<u8> { self.1 }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Error for NulError {
    fn description(&self) -> &str { "nul byte found in data" }
}

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nul byte found in provided data at position: {}", self.0)
    }
}

impl IntoStringError {
    /// Consumes this error, returning original `CString` which generated the
    /// error.
    #[stable(feature = "cstring_into", since = "1.7.0")]
    pub fn into_cstring(self) -> CString {
        self.inner
    }

    /// Access the underlying UTF-8 error that was the cause of this error.
    #[stable(feature = "cstring_into", since = "1.7.0")]
    pub fn utf8_error(&self) -> Utf8Error {
        self.error
    }
}

impl Error for IntoStringError {
    fn description(&self) -> &str {
        "C string contained non-utf8 bytes"
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.error)
    }
}

#[stable(feature = "cstring_into", since = "1.7.0")]
impl fmt::Display for IntoStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl CStr {
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a CStr {
        let len = libc::strlen(ptr);
        mem::transmute(slice::from_raw_parts(ptr, len as usize + 1))
    }

    pub fn from_bytes_with_nul(bytes: &[u8])
                               -> Result<&CStr, FromBytesWithNulError> {
        if bytes.is_empty() || memchr::fallback::memchr(0, &bytes) != Some(bytes.len() - 1) {
            Err(FromBytesWithNulError { _a: () })
        } else {
            Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
        }
    }

    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &CStr {
        mem::transmute(bytes)
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    pub fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { mem::transmute(&self.inner) }
    }

    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        // NB: When CStr is changed to perform the length check in .to_bytes()
        // instead of in from_ptr(), it may be worth considering if this should
        // be rewritten to do the UTF-8 check inline with the length calculation
        // instead of doing it afterwards.
        str::from_utf8(self.to_bytes())
    }

    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.to_bytes())
    }
}

impl PartialEq for CStr {
    fn eq(&self, other: &CStr) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}
impl Eq for CStr {}
impl PartialOrd for CStr {
    fn partial_cmp(&self, other: &CStr) -> Option<Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}
impl Ord for CStr {
    fn cmp(&self, other: &CStr) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl ToOwned for CStr {
    type Owned = CString;

    fn to_owned(&self) -> CString {
        unsafe { CString::from_vec_unchecked(self.to_bytes().to_vec()) }
    }
}

impl<'a> From<&'a CStr> for CString {
    fn from(s: &'a CStr) -> CString {
        s.to_owned()
    }
}

impl ops::Index<ops::RangeFull> for CString {
    type Output = CStr;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &CStr {
        self
    }
}

impl AsRef<CStr> for CStr {
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl AsRef<CStr> for CString {
    fn as_ref(&self) -> &CStr {
        self
    }
}
