// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Platform-independent platform abstraction
//!
//! This is the platform-independent portion of the standard libraries
//! platform abstraction layer, whereas `std::sys` is the
//! platform-specific portion.
//!
//! The relationship between `std::sys_common`, `std::sys` and the
//! rest of `std` is complex, with dependencies going in all
//! directions: `std` depending on `sys_common`, `sys_common`
//! depending on `sys`, and `sys` depending on `sys_common` and `std`.
//! Ideally `sys_common` would be split into two and the dependencies
//! between them all would form a dag, facilitating the extraction of
//! `std::sys` from the standard library.

#![allow(missing_docs)]
#![unstable(feature = "pal", reason = "unstable", issue = "0")]

use sync::Once;
use sys;

pub mod at_exit_imp;
#[cfg(any(not(cargobuild), feature = "backtrace"))]
pub mod backtrace;
pub mod condvar;
pub mod io;
pub mod mutex;
pub mod net;
pub mod poison;
pub mod remutex;
pub mod rwlock;
pub mod thread;
pub mod thread_info;
pub mod thread_local;
pub mod util;

#[cfg(any(not(cargobuild), feature = "backtrace"))]
#[cfg(any(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "emscripten"))),
          all(windows, target_env = "gnu")))]
pub mod gnu;

pub use pal_common::{AsInner, AsInnerMut, IntoInner, FromInner};
pub use self::at_exit_imp::at_exit;

macro_rules! rtabort {
    ($($t:tt)*) => (::sys_common::util::abort(format_args!($($t)*)))
}

/// One-time runtime cleanup.
pub fn cleanup() {
    static CLEANUP: Once = Once::new();
    CLEANUP.call_once(|| unsafe {
        sys::args::cleanup();
        sys::stack_overflow::cleanup();
        at_exit_imp::cleanup();
    });
}

// Computes (value*numer)/denom without overflow, as long as both
// (numer*denom) and the overall result fit into i64 (which is the case
// for our time conversions).
#[allow(dead_code)] // not used on all platforms
pub fn mul_div_u64(value: u64, numer: u64, denom: u64) -> u64 {
    let q = value / denom;
    let r = value % denom;
    // Decompose value as (value/denom*denom + value%denom),
    // substitute into (value*numer)/denom and simplify.
    // r < denom, so (denom*numer) is the upper bound of (r*numer)
    q * numer + r * numer / denom
}

#[test]
fn test_muldiv() {
    assert_eq!(mul_div_u64( 1_000_000_000_001, 1_000_000_000, 1_000_000),
               1_000_000_000_001_000);
}
