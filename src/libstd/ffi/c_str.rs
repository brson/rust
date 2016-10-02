// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use borrow::{Cow, Borrow};
use cmp::Ordering;
use fmt;
use mem;
use memchr;
use ops;
use os::raw::c_char;
use str;

// The CString and CStr types wrap the types from the c_str cate, except
// that they specialize the memchr operations
use c_str::CString as PCString;
use c_str::CStr as PCStr;

#[stable(feature = "rust1", since = "1.0.0")]
pub use c_str::{NulError, IntoStringError};
#[stable(feature = "cstr_from_bytes", since = "1.10.0")]
pub use c_str::{FromBytesWithNulError};

/// A type representing an owned C-compatible string
///
/// This type serves the primary purpose of being able to safely generate a
/// C-compatible string from a Rust byte slice or vector. An instance of this
/// type is a static guarantee that the underlying bytes contain no interior 0
/// bytes and the final byte is 0.
///
/// A `CString` is created from either a byte slice or a byte vector. After
/// being created, a `CString` predominately inherits all of its methods from
/// the `Deref` implementation to `[c_char]`. Note that the underlying array
/// is represented as an array of `c_char` as opposed to `u8`. A `u8` slice
/// can be obtained with the `as_bytes` method.  Slices produced from a `CString`
/// do *not* contain the trailing nul terminator unless otherwise specified.
///
/// # Examples
///
/// ```no_run
/// # fn main() {
/// use std::ffi::CString;
/// use std::os::raw::c_char;
///
/// extern {
///     fn my_printer(s: *const c_char);
/// }
///
/// let c_to_print = CString::new("Hello, world!").unwrap();
/// unsafe {
///     my_printer(c_to_print.as_ptr());
/// }
/// # }
/// ```
///
/// # Safety
///
/// `CString` is intended for working with traditional C-style strings
/// (a sequence of non-null bytes terminated by a single null byte); the
/// primary use case for these kinds of strings is interoperating with C-like
/// code. Often you will need to transfer ownership to/from that external
/// code. It is strongly recommended that you thoroughly read through the
/// documentation of `CString` before use, as improper ownership management
/// of `CString` instances can lead to invalid memory accesses, memory leaks,
/// and other memory errors.

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct CString {
    inner: PCString
}

/// Representation of a borrowed C string.
///
/// This dynamically sized type is only safely constructed via a borrowed
/// version of an instance of `CString`. This type can be constructed from a raw
/// C string as well and represents a C string borrowed from another location.
///
/// Note that this structure is **not** `repr(C)` and is not recommended to be
/// placed in the signatures of FFI functions. Instead safe wrappers of FFI
/// functions may leverage the unsafe `from_ptr` constructor to provide a safe
/// interface to other consumers.
///
/// # Examples
///
/// Inspecting a foreign C string
///
/// ```no_run
/// use std::ffi::CStr;
/// use std::os::raw::c_char;
///
/// extern { fn my_string() -> *const c_char; }
///
/// unsafe {
///     let slice = CStr::from_ptr(my_string());
///     println!("string length: {}", slice.to_bytes().len());
/// }
/// ```
///
/// Passing a Rust-originating C string
///
/// ```no_run
/// use std::ffi::{CString, CStr};
/// use std::os::raw::c_char;
///
/// fn work(data: &CStr) {
///     extern { fn work_with(data: *const c_char); }
///
///     unsafe { work_with(data.as_ptr()) }
/// }
///
/// let s = CString::new("data data data data").unwrap();
/// work(&s);
/// ```
///
/// Converting a foreign C string into a Rust `String`
///
/// ```no_run
/// use std::ffi::CStr;
/// use std::os::raw::c_char;
///
/// extern { fn my_string() -> *const c_char; }
///
/// fn my_string_safe() -> String {
///     unsafe {
///         CStr::from_ptr(my_string()).to_string_lossy().into_owned()
///     }
/// }
///
/// println!("string: {}", my_string_safe());
/// ```
#[derive(Hash)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct CStr {
    inner: PCStr
}

impl CString {
    #[inline]
    fn p(inner: PCString) -> CString {
        CString { inner: inner }
    }

    /// Creates a new C-compatible string from a container of bytes.
    ///
    /// This method will consume the provided data and use the underlying bytes
    /// to construct a new string, ensuring that there is a trailing 0 byte.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::ffi::CString;
    /// use std::os::raw::c_char;
    ///
    /// extern { fn puts(s: *const c_char); }
    ///
    /// let to_print = CString::new("Hello!").unwrap();
    /// unsafe {
    ///     puts(to_print.as_ptr());
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the bytes yielded contain an
    /// internal 0 byte. The error returned will contain the bytes as well as
    /// the position of the nul byte.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<CString, NulError> {
        Self::_new(t.into())
    }

    #[inline]
    fn _new(bytes: Vec<u8>) -> Result<CString, NulError> {
        use c_str::_new_nul_error;
        match memchr::memchr(0, &bytes) {
            Some(i) => Err(_new_nul_error(i, bytes)),
            None => Ok(unsafe { CString::p(PCString::from_vec_unchecked(bytes)) }),
        }
    }

    /// Creates a C-compatible string from a byte vector without checking for
    /// interior 0 bytes.
    ///
    /// This method is equivalent to `new` except that no runtime assertion
    /// is made that `v` contains no 0 bytes, and it requires an actual
    /// byte vector, not anything that can be converted to one with Into.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::CString;
    ///
    /// let raw = b"foo".to_vec();
    /// unsafe {
    ///     let c_string = CString::from_vec_unchecked(raw);
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub unsafe fn from_vec_unchecked(v: Vec<u8>) -> CString {
        CString::p(PCString::from_vec_unchecked(v))
    }

    /// Retakes ownership of a `CString` that was transferred to C.
    ///
    /// Additionally, the length of the string will be recalculated from the pointer.
    ///
    /// # Safety
    ///
    /// This should only ever be called with a pointer that was earlier
    /// obtained by calling `into_raw` on a `CString`. Other usage (e.g. trying to take
    /// ownership of a string that was allocated by foreign code) is likely to lead
    /// to undefined behavior or allocator corruption.
    #[stable(feature = "cstr_memory", since = "1.4.0")]
    #[inline]
    pub unsafe fn from_raw(ptr: *mut c_char) -> CString {
        CString::p(PCString::from_raw(ptr))
    }

    /// Transfers ownership of the string to a C caller.
    ///
    /// The pointer must be returned to Rust and reconstituted using
    /// `from_raw` to be properly deallocated. Specifically, one
    /// should *not* use the standard C `free` function to deallocate
    /// this string.
    ///
    /// Failure to call `from_raw` will lead to a memory leak.
    #[stable(feature = "cstr_memory", since = "1.4.0")]
    #[inline]
    pub fn into_raw(self) -> *mut c_char {
        self.inner.into_raw()
    }

    /// Converts the `CString` into a `String` if it contains valid Unicode data.
    ///
    /// On failure, ownership of the original `CString` is returned.
    #[stable(feature = "cstring_into", since = "1.7.0")]
    #[inline]
    pub fn into_string(self) -> Result<String, IntoStringError> {
        self.inner.into_string()
    }

    /// Returns the underlying byte buffer.
    ///
    /// The returned buffer does **not** contain the trailing nul separator and
    /// it is guaranteed to not have any interior nul bytes.
    #[stable(feature = "cstring_into", since = "1.7.0")]
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_bytes()
    }

    /// Equivalent to the `into_bytes` function except that the returned vector
    /// includes the trailing nul byte.
    #[stable(feature = "cstring_into", since = "1.7.0")]
    #[inline]
    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.inner.into_bytes_with_nul()
    }

    /// Returns the contents of this `CString` as a slice of bytes.
    ///
    /// The returned slice does **not** contain the trailing nul separator and
    /// it is guaranteed to not have any interior nul bytes.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    /// Equivalent to the `as_bytes` function except that the returned slice
    /// includes the trailing nul byte.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        self.inner.as_bytes_with_nul()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ops::Deref for CString {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        CStr::p(self.inner.deref())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for CString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

#[stable(feature = "cstring_into", since = "1.7.0")]
impl From<CString> for Vec<u8> {
    #[inline]
    fn from(s: CString) -> Vec<u8> {
        Vec::from(s.inner)
    }
}

#[stable(feature = "cstr_debug", since = "1.3.0")]
impl fmt::Debug for CStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

#[stable(feature = "cstr_default", since = "1.10.0")]
impl<'a> Default for &'a CStr {
    #[inline]
    fn default() -> &'a CStr {
        CStr::p(Default::default())
    }
}

#[stable(feature = "cstr_default", since = "1.10.0")]
impl Default for CString {
    /// Creates an empty `CString`.
    #[inline]
    fn default() -> CString {
        CString::p(PCString::default())
    }
}

#[stable(feature = "cstr_borrow", since = "1.3.0")]
impl Borrow<CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr {
        self
    }
}

impl CStr {
    #[inline]
    fn p<'a>(inner: &'a PCStr) -> &'a CStr {
        unsafe { mem::transmute(inner) }
    }

    /// Casts a raw C string to a safe C string wrapper.
    ///
    /// This function will cast the provided `ptr` to the `CStr` wrapper which
    /// allows inspection and interoperation of non-owned C strings. This method
    /// is unsafe for a number of reasons:
    ///
    /// * There is no guarantee to the validity of `ptr`
    /// * The returned lifetime is not guaranteed to be the actual lifetime of
    ///   `ptr`
    /// * There is no guarantee that the memory pointed to by `ptr` contains a
    ///   valid nul terminator byte at the end of the string.
    ///
    /// > **Note**: This operation is intended to be a 0-cost cast but it is
    /// > currently implemented with an up-front calculation of the length of
    /// > the string. This is not guaranteed to always be the case.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() {
    /// use std::ffi::CStr;
    /// use std::os::raw::c_char;
    ///
    /// extern {
    ///     fn my_string() -> *const c_char;
    /// }
    ///
    /// unsafe {
    ///     let slice = CStr::from_ptr(my_string());
    ///     println!("string returned: {}", slice.to_str().unwrap());
    /// }
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a CStr {
        CStr::p(PCStr::from_ptr(ptr))
    }

    /// Creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `CStr` wrapper after
    /// ensuring that it is null terminated and does not contain any interior
    /// nul bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::CStr;
    ///
    /// let cstr = CStr::from_bytes_with_nul(b"hello\0");
    /// assert!(cstr.is_ok());
    /// ```
    #[stable(feature = "cstr_from_bytes", since = "1.10.0")]
    #[inline]
    pub fn from_bytes_with_nul(bytes: &[u8])
                               -> Result<&CStr, FromBytesWithNulError> {
        use c_str::_new_from_bytes_with_nul_error;
        if bytes.is_empty() || memchr::memchr(0, &bytes) != Some(bytes.len() - 1) {
            Err(_new_from_bytes_with_nul_error())
        } else {
            Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
        }
    }

    /// Unsafely creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `CStr` wrapper without
    /// performing any sanity checks. The provided slice must be null terminated
    /// and not contain any interior nul bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::{CStr, CString};
    ///
    /// unsafe {
    ///     let cstring = CString::new("hello").unwrap();
    ///     let cstr = CStr::from_bytes_with_nul_unchecked(cstring.to_bytes_with_nul());
    ///     assert_eq!(cstr, &*cstring);
    /// }
    /// ```
    #[stable(feature = "cstr_from_bytes", since = "1.10.0")]
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &CStr {
        CStr::p(PCStr::from_bytes_with_nul_unchecked(bytes))
    }

    /// Returns the inner pointer to this C string.
    ///
    /// The returned pointer will be valid for as long as `self` is and points
    /// to a contiguous region of memory terminated with a 0 byte to represent
    /// the end of the string.
    ///
    /// **WARNING**
    ///
    /// It is your responsibility to make sure that the underlying memory is not
    /// freed too early. For example, the following code will cause undefined
    /// behaviour when `ptr` is used inside the `unsafe` block:
    ///
    /// ```no_run
    /// use std::ffi::{CString};
    ///
    /// let ptr = CString::new("Hello").unwrap().as_ptr();
    /// unsafe {
    ///     // `ptr` is dangling
    ///     *ptr;
    /// }
    /// ```
    ///
    /// This happens because the pointer returned by `as_ptr` does not carry any
    /// lifetime information and the string is deallocated immediately after
    /// the `CString::new("Hello").unwrap().as_ptr()` expression is evaluated.
    /// To fix the problem, bind the string to a local variable:
    ///
    /// ```no_run
    /// use std::ffi::{CString};
    ///
    /// let hello = CString::new("Hello").unwrap();
    /// let ptr = hello.as_ptr();
    /// unsafe {
    ///     // `ptr` is valid because `hello` is in scope
    ///     *ptr;
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    /// Converts this C string to a byte slice.
    ///
    /// This function will calculate the length of this string (which normally
    /// requires a linear amount of work to be done) and then return the
    /// resulting slice of `u8` elements.
    ///
    /// The returned slice will **not** contain the trailing nul that this C
    /// string has.
    ///
    /// > **Note**: This method is currently implemented as a 0-cost cast, but
    /// > it is planned to alter its definition in the future to perform the
    /// > length calculation whenever this method is called.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        self.inner.to_bytes()
    }

    /// Converts this C string to a byte slice containing the trailing 0 byte.
    ///
    /// This function is the equivalent of `to_bytes` except that it will retain
    /// the trailing nul instead of chopping it off.
    ///
    /// > **Note**: This method is currently implemented as a 0-cost cast, but
    /// > it is planned to alter its definition in the future to perform the
    /// > length calculation whenever this method is called.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        self.inner.to_bytes_with_nul()
    }

    /// Yields a `&str` slice if the `CStr` contains valid UTF-8.
    ///
    /// This function will calculate the length of this string and check for
    /// UTF-8 validity, and then return the `&str` if it's valid.
    ///
    /// > **Note**: This method is currently implemented to check for validity
    /// > after a 0-cost cast, but it is planned to alter its definition in the
    /// > future to perform the length calculation in addition to the UTF-8
    /// > check whenever this method is called.
    #[stable(feature = "cstr_to_str", since = "1.4.0")]
    #[inline]
    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        self.inner.to_str()
    }

    /// Converts a `CStr` into a `Cow<str>`.
    ///
    /// This function will calculate the length of this string (which normally
    /// requires a linear amount of work to be done) and then return the
    /// resulting slice as a `Cow<str>`, replacing any invalid UTF-8 sequences
    /// with `U+FFFD REPLACEMENT CHARACTER`.
    ///
    /// > **Note**: This method is currently implemented to check for validity
    /// > after a 0-cost cast, but it is planned to alter its definition in the
    /// > future to perform the length calculation in addition to the UTF-8
    /// > check whenever this method is called.
    #[stable(feature = "cstr_to_str", since = "1.4.0")]
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<str> {
        self.inner.to_string_lossy()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl PartialEq for CStr {
    #[inline]
    fn eq(&self, other: &CStr) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Eq for CStr {}
#[stable(feature = "rust1", since = "1.0.0")]
impl PartialOrd for CStr {
    #[inline]
    fn partial_cmp(&self, other: &CStr) -> Option<Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Ord for CStr {
    #[inline]
    fn cmp(&self, other: &CStr) -> Ordering {
        self.to_bytes().cmp(&other.to_bytes())
    }
}

#[stable(feature = "cstr_borrow", since = "1.3.0")]
impl ToOwned for CStr {
    type Owned = CString;

    #[inline]
    fn to_owned(&self) -> CString {
        CString::p(self.inner.to_owned())
    }
}

#[stable(feature = "cstring_asref", since = "1.7.0")]
impl<'a> From<&'a CStr> for CString {
    #[inline]
    fn from(s: &'a CStr) -> CString {
        CString::p(PCString::from(&s.inner))
    }
}

#[stable(feature = "cstring_asref", since = "1.7.0")]
impl ops::Index<ops::RangeFull> for CString {
    type Output = CStr;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &CStr {
        self
    }
}

#[stable(feature = "cstring_asref", since = "1.7.0")]
impl AsRef<CStr> for CStr {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

#[stable(feature = "cstring_asref", since = "1.7.0")]
impl AsRef<CStr> for CString {
    fn as_ref(&self) -> &CStr {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os::raw::c_char;
    use borrow::Cow::{Borrowed, Owned};
    use hash::{SipHasher, Hash, Hasher};

    #[test]
    fn c_to_rust() {
        let data = b"123\0";
        let ptr = data.as_ptr() as *const c_char;
        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_bytes(), b"123");
            assert_eq!(CStr::from_ptr(ptr).to_bytes_with_nul(), b"123\0");
        }
    }

    #[test]
    fn simple() {
        let s = CString::new("1234").unwrap();
        assert_eq!(s.as_bytes(), b"1234");
        assert_eq!(s.as_bytes_with_nul(), b"1234\0");
    }

    #[test]
    fn build_with_zero1() {
        assert!(CString::new(&b"\0"[..]).is_err());
    }
    #[test]
    fn build_with_zero2() {
        assert!(CString::new(vec![0]).is_err());
    }

    #[test]
    fn build_with_zero3() {
        unsafe {
            let s = CString::from_vec_unchecked(vec![0]);
            assert_eq!(s.as_bytes(), b"\0");
        }
    }

    #[test]
    fn formatted() {
        let s = CString::new(&b"abc\x01\x02\n\xE2\x80\xA6\xFF"[..]).unwrap();
        assert_eq!(format!("{:?}", s), r#""abc\x01\x02\n\xe2\x80\xa6\xff""#);
    }

    #[test]
    fn borrowed() {
        unsafe {
            let s = CStr::from_ptr(b"12\0".as_ptr() as *const _);
            assert_eq!(s.to_bytes(), b"12");
            assert_eq!(s.to_bytes_with_nul(), b"12\0");
        }
    }

    #[test]
    fn to_str() {
        let data = b"123\xE2\x80\xA6\0";
        let ptr = data.as_ptr() as *const c_char;
        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str(), Ok("123…"));
            assert_eq!(CStr::from_ptr(ptr).to_string_lossy(), Borrowed("123…"));
        }
        let data = b"123\xE2\0";
        let ptr = data.as_ptr() as *const c_char;
        unsafe {
            assert!(CStr::from_ptr(ptr).to_str().is_err());
            assert_eq!(CStr::from_ptr(ptr).to_string_lossy(), Owned::<str>(format!("123\u{FFFD}")));
        }
    }

    #[test]
    fn to_owned() {
        let data = b"123\0";
        let ptr = data.as_ptr() as *const c_char;

        let owned = unsafe { CStr::from_ptr(ptr).to_owned() };
        assert_eq!(owned.as_bytes_with_nul(), data);
    }

    #[test]
    fn equal_hash() {
        let data = b"123\xE2\xFA\xA6\0";
        let ptr = data.as_ptr() as *const c_char;
        let cstr: &'static CStr = unsafe { CStr::from_ptr(ptr) };

        let mut s = SipHasher::new_with_keys(0, 0);
        cstr.hash(&mut s);
        let cstr_hash = s.finish();
        let mut s = SipHasher::new_with_keys(0, 0);
        CString::new(&data[..data.len() - 1]).unwrap().hash(&mut s);
        let cstring_hash = s.finish();

        assert_eq!(cstr_hash, cstring_hash);
    }

    #[test]
    fn from_bytes_with_nul() {
        let data = b"123\0";
        let cstr = CStr::from_bytes_with_nul(data);
        assert_eq!(cstr.map(CStr::to_bytes), Ok(&b"123"[..]));
        let cstr = CStr::from_bytes_with_nul(data);
        assert_eq!(cstr.map(CStr::to_bytes_with_nul), Ok(&b"123\0"[..]));

        unsafe {
            let cstr = CStr::from_bytes_with_nul(data);
            let cstr_unchecked = CStr::from_bytes_with_nul_unchecked(data);
            assert_eq!(cstr, Ok(cstr_unchecked));
        }
    }

    #[test]
    fn from_bytes_with_nul_unterminated() {
        let data = b"123";
        let cstr = CStr::from_bytes_with_nul(data);
        assert!(cstr.is_err());
    }

    #[test]
    fn from_bytes_with_nul_interior() {
        let data = b"1\023\0";
        let cstr = CStr::from_bytes_with_nul(data);
        assert!(cstr.is_err());
    }
}
