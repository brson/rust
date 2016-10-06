// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::f64::{NAN, NEG_INFINITY};
use core::intrinsics;
use libc::c_int;

// Solaris/Illumos requires a wrapper around log, log2, and log10 functions
// because of their non-standard behavior (e.g. log(-n) returns -Inf instead
// of expected NaN).
#[inline]
pub fn log_wrapper<F: Fn(f64) -> f64>(v: f64, log_fn: F) -> f64 {
    if !cfg!(target_os = "solaris") {
        log_fn(v)
    } else {
        if v.is_finite() {
            if v > 0.0 {
                log_fn(v)
            } else if v == 0.0 {
                NEG_INFINITY // log(0) = -Inf
            } else {
                NAN // log(-n) = NaN
            }
        } else if v.is_nan() {
            v // log(NaN) = NaN
        } else if v > 0.0 {
            v // log(Inf) = Inf
        } else {
            NAN // log(-Inf) = NaN
        }
    }
}

#[inline]
pub fn log2(v: f64) -> f64 {
    log_wrapper(v, |n| {
        #[cfg(target_os = "android")]
        return ::sys::android::log2f64(n);
        #[cfg(not(target_os = "android"))]
        return unsafe { intrinsics::log2f64(n) };
    })
}

#[inline]
pub fn ldexp(x: f64, exp: isize) -> f64 {
    unsafe { cmath::ldexp(x, exp as c_int) }
}

#[inline]
pub fn frexp(v: f64) -> (f64, isize) {
    unsafe {
        let mut exp = 0;
        let x = cmath::frexp(v, &mut exp);
        (x, exp as isize)
    }
}

#[inline]
pub fn next_after(v: f64, other: f64) -> f64 {
    unsafe { cmath::nextafter(v, other) }
}

#[inline]
pub fn max(v: f64, other: f64) -> f64 {
    unsafe { cmath::fmax(v, other) }
}

#[inline]
pub fn min(v: f64, other: f64) -> f64 {
    unsafe { cmath::fmin(v, other) }
}

#[inline]
pub fn abs_sub(v: f64, other: f64) -> f64 {
    unsafe { cmath::fdim(v, other) }
}

#[inline]
pub fn cbrt(v: f64) -> f64 {
    unsafe { cmath::cbrt(v) }
}

#[inline]
pub fn hypot(v: f64, other: f64) -> f64 {
    unsafe { cmath::hypot(v, other) }
}

#[inline]
pub fn tan(v: f64) -> f64 {
    unsafe { cmath::tan(v) }
}

#[inline]
pub fn asin(v: f64) -> f64 {
    unsafe { cmath::asin(v) }
}

#[inline]
pub fn acos(v: f64) -> f64 {
    unsafe { cmath::acos(v) }
}

#[inline]
pub fn atan(v: f64) -> f64 {
    unsafe { cmath::atan(v) }
}

#[inline]
pub fn atan2(v: f64, other: f64) -> f64 {
    unsafe { cmath::atan2(v, other) }
}

#[inline]
pub fn exp_m1(v: f64) -> f64 {
    unsafe { cmath::expm1(v) }
}

#[inline]
pub fn ln_1p(v: f64) -> f64 {
    unsafe { cmath::log1p(v) }
}

#[inline]
pub fn sinh(v: f64) -> f64 {
    unsafe { cmath::sinh(v) }
}

#[inline]
pub fn cosh(v: f64) -> f64 {
    unsafe { cmath::cosh(v) }
}

#[inline]
pub fn tanh(v: f64) -> f64 {
    unsafe { cmath::tanh(v) }
}

mod cmath {
    use libc::{c_double, c_int};

    #[link_name = "m"]
    extern {
        pub fn acos(n: c_double) -> c_double;
        pub fn asin(n: c_double) -> c_double;
        pub fn atan(n: c_double) -> c_double;
        pub fn atan2(a: c_double, b: c_double) -> c_double;
        pub fn cbrt(n: c_double) -> c_double;
        pub fn cosh(n: c_double) -> c_double;
        pub fn expm1(n: c_double) -> c_double;
        pub fn fdim(a: c_double, b: c_double) -> c_double;
        pub fn fmax(a: c_double, b: c_double) -> c_double;
        pub fn fmin(a: c_double, b: c_double) -> c_double;
        pub fn frexp(n: c_double, value: &mut c_int) -> c_double;
        pub fn ldexp(x: c_double, n: c_int) -> c_double;
        pub fn log1p(n: c_double) -> c_double;
        pub fn nextafter(x: c_double, y: c_double) -> c_double;
        pub fn sinh(n: c_double) -> c_double;
        pub fn tan(n: c_double) -> c_double;
        pub fn tanh(n: c_double) -> c_double;
        pub fn hypot(x: c_double, y: c_double) -> c_double;
    }
}

