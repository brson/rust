// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use collections::borrow::ToOwned;
use collections::String;
use core::str;
use c_str::CStr;
use libc::{self, c_int, c_char};

const TMPBUF_SZ: usize = 128;

extern {
    #[cfg(not(target_os = "dragonfly"))]
    #[cfg_attr(any(target_os = "linux", target_os = "emscripten"),
               link_name = "__errno_location")]
    #[cfg_attr(any(target_os = "bitrig",
                   target_os = "netbsd",
                   target_os = "openbsd",
                   target_os = "android",
                   target_env = "newlib"),
               link_name = "__errno")]
    #[cfg_attr(target_os = "solaris", link_name = "___errno")]
    #[cfg_attr(any(target_os = "macos",
                   target_os = "ios",
                   target_os = "freebsd"),
               link_name = "__error")]
    #[cfg_attr(target_os = "haiku", link_name = "_errnop")]
    fn errno_location() -> *mut c_int;
}

/// Returns the platform-specific value of errno
#[cfg(not(target_os = "dragonfly"))]
pub fn errno() -> i32 {
    unsafe {
        (*errno_location()) as i32
    }
}

/// Sets the platform-specific value of errno
#[cfg(target_os = "solaris")] // only needed for readdir so far
pub fn set_errno(e: i32) {
    unsafe {
        *errno_location() = e as c_int
    }
}

#[cfg(target_os = "dragonfly")]
pub fn errno() -> i32 {
    extern {
        #[thread_local]
        static errno: c_int;
    }

    errno as i32
}

/// Gets a detailed string description for the given error number.
pub fn error_string(errno: i32) -> String {
    extern {
        #[cfg_attr(any(target_os = "linux", target_env = "newlib"),
                   link_name = "__xpg_strerror_r")]
        fn strerror_r(errnum: c_int, buf: *mut c_char,
                      buflen: libc::size_t) -> c_int;
    }

    let mut buf = [0 as c_char; TMPBUF_SZ];

    let p = buf.as_mut_ptr();
    unsafe {
        if strerror_r(errno as c_int, p, buf.len() as libc::size_t) < 0 {
            panic!("strerror_r failure");
        }

        let p = p as *const _;
        str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap().to_owned()
    }
}
