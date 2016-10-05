// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::intrinsics;
use libc::c_int;

#[inline]
pub fn floor(v: f32) -> f32 {
    unsafe { intrinsics::floorf32(v) }
}

#[inline]
pub fn ceil(v: f32) -> f32 {
    unsafe { intrinsics::ceilf32(v) }
}

#[inline]
pub fn powf(v: f32, n: f32) -> f32 {
    unsafe { intrinsics::powf32(v, n) }
}

#[inline]
pub fn exp(v: f32) -> f32 {
    unsafe { intrinsics::expf32(v) }
}

#[inline]
pub fn ln(v: f32) -> f32 {
    unsafe { intrinsics::logf32(v) }
}

#[inline]
pub fn log2(v: f32) -> f32 {
    unsafe { intrinsics::log2f32(v) }
}

#[inline]
pub fn log10(v: f32) -> f32 {
    unsafe { intrinsics::log10f32(v) }
}

#[inline]
pub fn ldexp(x: f32, exp: isize) -> f32 {
    unsafe { cmath::ldexpf(x, exp as c_int) }
}

#[inline]
pub fn frexp(v: f32) -> (f32, isize) {
    unsafe {
        let mut exp = 0;
        let x = cmath::frexpf(v, &mut exp);
        (x, exp as isize)
    }
}

#[inline]
pub fn next_after(v: f32, other: f32) -> f32 {
    unsafe { cmath::nextafterf(v, other) }
}

#[inline]
pub fn max(v: f32, other: f32) -> f32 {
    unsafe { cmath::fmaxf(v, other) }
}

#[inline]
pub fn min(v: f32, other: f32) -> f32 {
    unsafe { cmath::fminf(v, other) }
}

#[inline]
pub fn abs_sub(v: f32, other: f32) -> f32 {
    unsafe { cmath::fdimf(v, other) }
}

#[inline]
pub fn cbrt(v: f32) -> f32 {
    unsafe { cmath::cbrtf(v) }
}

#[inline]
pub fn hypot(v: f32, other: f32) -> f32 {
    unsafe { cmath::hypotf(v, other) }
}

#[inline]
pub fn sin(v: f32) -> f32 {
    unsafe { intrinsics::sinf32(v) }
}

#[inline]
pub fn cos(v: f32) -> f32 {
    unsafe { intrinsics::cosf32(v) }
}

#[inline]
pub fn tan(v: f32) -> f32 {
    unsafe { cmath::tanf(v) }
}

#[inline]
pub fn asin(v: f32) -> f32 {
    unsafe { cmath::asinf(v) }
}

#[inline]
pub fn acos(v: f32) -> f32 {
    unsafe { cmath::acosf(v) }
}

#[inline]
pub fn atan(v: f32) -> f32 {
    unsafe { cmath::atanf(v) }
}

#[inline]
pub fn atan2(v: f32, other: f32) -> f32 {
    unsafe { cmath::atan2f(v, other) }
}

#[inline]
pub fn exp_m1(v: f32) -> f32 {
    unsafe { cmath::expm1f(v) }
}

#[inline]
pub fn ln_1p(v: f32) -> f32 {
    unsafe { cmath::log1pf(v) }
}

#[inline]
pub fn sinh(v: f32) -> f32 {
    unsafe { cmath::sinhf(v) }
}

#[inline]
pub fn cosh(v: f32) -> f32 {
    unsafe { cmath::coshf(v) }
}

#[inline]
pub fn tanh(v: f32) -> f32 {
    unsafe { cmath::tanhf(v) }
}

mod cmath {
    use libc::{c_float, c_int};

    extern {
        pub fn cbrtf(n: c_float) -> c_float;
        pub fn expm1f(n: c_float) -> c_float;
        pub fn fdimf(a: c_float, b: c_float) -> c_float;
        pub fn fmaxf(a: c_float, b: c_float) -> c_float;
        pub fn fminf(a: c_float, b: c_float) -> c_float;
        pub fn log1pf(n: c_float) -> c_float;
        pub fn nextafterf(x: c_float, y: c_float) -> c_float;
        pub fn acosf(n: c_float) -> c_float;
        pub fn asinf(n: c_float) -> c_float;
        pub fn atan2f(a: c_float, b: c_float) -> c_float;
        pub fn atanf(n: c_float) -> c_float;
        pub fn coshf(n: c_float) -> c_float;
        pub fn frexpf(n: c_float, value: &mut c_int) -> c_float;
        pub fn ldexpf(x: c_float, n: c_int) -> c_float;
        pub fn sinhf(n: c_float) -> c_float;
        pub fn tanf(n: c_float) -> c_float;
        pub fn tanhf(n: c_float) -> c_float;
        pub fn hypotf(x: c_float, y: c_float) -> c_float;
    }
}

