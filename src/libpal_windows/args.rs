// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)] // runtime init functions not used during testing
#![allow(non_snake_case)]

use os::c;
use core::slice;
use core::ops::Range;
use os_str;
use libc::{c_int, c_void};
use os::wtf8::Wtf8Buf;

pub unsafe fn init(_argc: isize, _argv: *const *const u8) { }

pub unsafe fn cleanup() { }

pub fn args() -> Args {
    unsafe {
        let mut nArgs: c_int = 0;
        let lpCmdLine = c::GetCommandLineW();
        let szArgList = c::CommandLineToArgvW(lpCmdLine, &mut nArgs);

        // szArcList can be NULL if CommandLinToArgvW failed,
        // but in that case nArgs is 0 so we won't actually
        // try to read a null pointer
        Args { cur: szArgList, range: 0..(nArgs as isize) }
    }
}

pub struct Args {
    range: Range<isize>,
    cur: *mut *mut u16,
}

unsafe fn os_string_from_ptr(ptr: *mut u16) -> os_str::Buf {
    let mut len = 0;
    while *ptr.offset(len) != 0 { len += 1; }

    // Push it onto the list.
    let ptr = ptr as *const u16;
    let buf = slice::from_raw_parts(ptr, len as usize);
    os_str::Buf { inner: Wtf8Buf::from_wide(buf) }
}

impl Iterator for Args {
    type Item = os_str::Buf;
    fn next(&mut self) -> Option<os_str::Buf> {
        self.range.next().map(|i| unsafe { os_string_from_ptr(*self.cur.offset(i)) } )
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.range.size_hint() }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<os_str::Buf> {
        self.range.next_back().map(|i| unsafe { os_string_from_ptr(*self.cur.offset(i)) } )
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize { self.range.len() }
}

impl Drop for Args {
    fn drop(&mut self) {
        // self.cur can be null if CommandLineToArgvW previously failed,
        // but LocalFree ignores NULL pointers
        unsafe { c::LocalFree(self.cur as *mut c_void); }
    }
}
