// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Android ABI-compatibility module
//!
//! The ABI of Android has changed quite a bit over time, and libstd attempts to
//! be both forwards and backwards compatible as much as possible. We want to
//! always work with the most recent version of Android, but we also want to
//! work with older versions of Android for whenever projects need to.
//!
//! Our current minimum supported Android version is `android-9`, e.g. Android
//! with API level 9. We then in theory want to work on that and all future
//! versions of Android!
//!
//! Some of the detection here is done at runtime via `dlopen` and
//! introspection. Other times no detection is performed at all and we just
//! provide a fallback implementation as some versions of Android we support
//! don't have the function.
//!
//! You'll find more details below about why each compatibility shim is needed.

#![cfg(target_os = "android")]

use libc::{c_int, sighandler_t};

use io;
use sys::cvt_r;

// The `log2` and `log2f` functions apparently appeared in android-18, or at
// least you can see they're not present in the android-17 header [1] and they
// are present in android-18 [2].
//
// [1]: https://chromium.googlesource.com/android_tools/+/20ee6d20/ndk/platforms
//                                       /android-17/arch-arm/usr/include/math.h
// [2]: https://chromium.googlesource.com/android_tools/+/20ee6d20/ndk/platforms
//                                       /android-18/arch-arm/usr/include/math.h
//
// Note that these shims are likely less precise than directly calling `log2`,
// but hopefully that should be enough for now...
//
// Note that mathematically, for any arbitrary `y`:
//
//      log_2(x) = log_y(x) / log_y(2)
//               = log_y(x) / (1 / log_2(y))
//               = log_y(x) * log_2(y)
//
// Hence because `ln` (log_e) is available on all Android we just choose `y = e`
// and get:
//
//      log_2(x) = ln(x) * log_2(e)

#[cfg(not(test))]
pub fn log2f32(f: f32) -> f32 {
    f.ln() * ::f32::consts::LOG2_E
}

#[cfg(not(test))]
pub fn log2f64(f: f64) -> f64 {
    f.ln() * ::f64::consts::LOG2_E
}

// The `ftruncate64` symbol apparently appeared in android-12, so we do some
// dynamic detection to see if we can figure out whether `ftruncate64` exists.
//
// If it doesn't we just fall back to `ftruncate`, generating an error for
// too-large values.
pub fn ftruncate64(fd: c_int, size: u64) -> io::Result<()> {
    weak!(fn ftruncate64(c_int, i64) -> c_int);

    extern {
        fn ftruncate(fd: c_int, off: i32) -> c_int;
    }

    unsafe {
        match ftruncate64.get() {
            Some(f) => cvt_r(|| f(fd, size as i64)).map(|_| ()),
            None => {
                if size > i32::max_value() as u64 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                                       "cannot truncate >2GB"))
                } else {
                    cvt_r(|| ftruncate(fd, size as i32)).map(|_| ())
                }
            }
        }
    }
}
