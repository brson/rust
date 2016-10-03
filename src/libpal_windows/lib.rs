// Copyright 2012-2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Platform abstraction layer for Windows

#![crate_name = "pal_windows"]
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
#![feature(asm)]
#![feature(box_syntax)]
#![feature(char_escape_debug)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(libc)]
#![feature(linkage)]
#![feature(pal)]
#![feature(question_mark)]
#![feature(repr_simd)]
#![feature(slice_patterns)]
#![feature(staged_api)]
#![feature(str_internals)]
#![feature(unicode)]

extern crate alloc;
#[macro_use]
extern crate collections;
extern crate c_str;
extern crate libc;
extern crate pal_common;
extern crate rustc_unicode;

pub mod args;
pub mod at_exit;
pub mod condvar;
pub mod env;
pub mod errno;
pub mod memchr;
pub mod mutex;
pub mod os_str;
pub mod rwlock;
pub mod thread_local;

// On Windows, use the processor-specific __fastfail mechanism.  In Windows 8
// and later, this will terminate the process immediately without running any
// in-process exception handlers.  In earlier versions of Windows, this
// sequence of instructions will be treated as an access violation,
// terminating the process but without necessarily bypassing all exception
// handlers.
//
// https://msdn.microsoft.com/en-us/library/dn774154.aspx
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn abort_internal() -> ! {
    asm!("int $$0x29" :: "{ecx}"(7) ::: volatile); // 7 is FAST_FAIL_FATAL_APP_EXIT
    core::intrinsics::unreachable();
}

// Platform-specific functions used by std::sys
pub mod os {
    use pal_common::duration::Duration;
    
    #[macro_use] pub mod compat;
    pub mod c;
    pub mod wtf8;

    pub fn dur2timeout(dur: Duration) -> c::DWORD {
        // Note that a duration is a (u64, u32) (seconds, nanoseconds) pair, and the
        // timeouts in windows APIs are typically u32 milliseconds. To translate, we
        // have two pieces to take care of:
        //
        // * Nanosecond precision is rounded up
        // * Greater than u32::MAX milliseconds (50 days) is rounded up to INFINITE
        //   (never time out).
        dur.as_secs().checked_mul(1000).and_then(|ms| {
            ms.checked_add((dur.subsec_nanos() as u64) / 1_000_000)
        }).and_then(|ms| {
            ms.checked_add(if dur.subsec_nanos() % 1_000_000 > 0 {1} else {0})
        }).map(|ms| {
            if ms > <c::DWORD>::max_value() as u64 {
                c::INFINITE
            } else {
                ms as c::DWORD
            }
        }).unwrap_or(c::INFINITE)
    }
}
