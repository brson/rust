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

#![feature(asm)]
#![feature(char_escape_debug)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(libc)]
#![feature(pal)]
#![feature(question_mark)]
#![feature(repr_simd)]
#![feature(slice_patterns)]
#![feature(staged_api)]
#![feature(str_internals)]
#![feature(unicode)]

extern crate collections;
extern crate c_str;
extern crate libc;
extern crate pal_common;
extern crate rustc_unicode;

pub mod os_str;

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
    #[macro_use] pub mod compat;
    pub mod c;
    pub mod wtf8;
}
