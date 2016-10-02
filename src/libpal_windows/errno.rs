// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(non_snake_case)]

use collections::String;
use core::ptr;
use os::c;

pub fn errno() -> i32 {
    unsafe { c::GetLastError() as i32 }
}

/// Gets a detailed string description for the given error number.
pub fn error_string(errnum: i32) -> String {
    // This value is calculated from the macro
    // MAKELANGID(LANG_SYSTEM_DEFAULT, SUBLANG_SYS_DEFAULT)
    let langId = 0x0800 as c::DWORD;

    let mut buf = [0 as c::WCHAR; 2048];

    unsafe {
        let res = c::FormatMessageW(c::FORMAT_MESSAGE_FROM_SYSTEM |
                                        c::FORMAT_MESSAGE_IGNORE_INSERTS,
                                    ptr::null_mut(),
                                    errnum as c::DWORD,
                                    langId,
                                    buf.as_mut_ptr(),
                                    buf.len() as c::DWORD,
                                    ptr::null()) as usize;
        if res == 0 {
            // Sometimes FormatMessageW can fail e.g. system doesn't like langId,
            let fm_err = errno();
            return format!("OS Error {} (FormatMessageW() returned error {})",
                           errnum, fm_err);
        }

        match String::from_utf16(&buf[..res]) {
            Ok(mut msg) => {
                // Trim trailing CRLF inserted by FormatMessageW
                let len = msg.trim_right().len();
                msg.truncate(len);
                msg
            },
            Err(..) => format!("OS Error {} (FormatMessageW() returned \
                                invalid UTF-16)", errnum),
        }
    }
}
