// Copyright 2012-2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Platform abstraction layer for Unix

#![crate_name = "pal_unix"]
#![unstable(feature = "platform_abstraction_layer", reason = "unstable", issue = "0")]
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
#![feature(core_intrinsics)]
#![feature(libc)]
#![feature(oom)]
#![feature(staged_api)]

extern crate alloc;
extern crate libc;

pub mod android;

pub fn init() {
    use alloc::oom;

    // By default, some platforms will send a *signal* when an EPIPE error
    // would otherwise be delivered. This runtime doesn't install a SIGPIPE
    // handler, causing it to kill the program, which isn't exactly what we
    // want!
    //
    // Hence, we set SIGPIPE to ignore when the program starts up in order
    // to prevent this problem.
    unsafe {
        reset_sigpipe();
    }

    oom::set_oom_handler(oom_handler);

    // A nicer handler for out-of-memory situations than the default one. This
    // one prints a message to stderr before aborting. It is critical that this
    // code does not allocate any memory since we are in an OOM situation. Any
    // errors are ignored while printing since there's nothing we can do about
    // them and we are about to exit anyways.
    fn oom_handler() -> ! {
        use core::intrinsics;
        let msg = "fatal runtime error: out of memory\n";
        unsafe {
            libc::write(libc::STDERR_FILENO,
                        msg.as_ptr() as *const libc::c_void,
                        msg.len() as libc::size_t);
            intrinsics::abort();
        }
    }

    #[cfg(not(any(target_os = "nacl", target_os = "emscripten")))]
    unsafe fn reset_sigpipe() {

        assert!(::os::signal(libc::SIGPIPE, libc::SIG_IGN) != !0);
    }
    #[cfg(any(target_os = "nacl", target_os = "emscripten"))]
    unsafe fn reset_sigpipe() {}
}

// Unix-specific stuff used by std::sys. Should become private eventually
pub mod os {
    #[cfg(target_os = "android")]
    pub use android::signal;
    #[cfg(not(target_os = "android"))]
    pub use libc::signal;
}
