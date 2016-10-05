// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(not(target_env = "msvc"))]
use core::intrinsics;
use libc::c_int;

#[inline]
pub fn floor(v: f32) -> f32 {
    // On MSVC LLVM will lower many math intrinsics to a call to the
    // corresponding function. On MSVC, however, many of these functions
    // aren't actually available as symbols to call, but rather they are all
    // `static inline` functions in header files. This means that from a C
    // perspective it's "compatible", but not so much from an ABI
    // perspective (which we're worried about).
    //
    // The inline header functions always just cast to a f64 and do their
    // operation, so we do that here as well, but only for MSVC targets.
    //
    // Note that there are many MSVC-specific float operations which
    // redirect to this comment, so `floorf` is just one case of a missing
    // function on MSVC, but there are many others elsewhere.
    #[cfg(target_env = "msvc")]
    return (v as f64).floor() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::floorf32(v) };
}

#[inline]
pub fn ceil(v: f32) -> f32 {
    // see notes above in `floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).ceil() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::ceilf32(v) };
}

#[inline]
pub fn powf(v: f32, n: f32) -> f32 {
    // see notes above in `floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).powf(n as f64) as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::powf32(v, n) };
}

#[inline]
pub fn exp(v: f32) -> f32 {
    // see notes above in `floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).exp() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::expf32(v) };
}

#[inline]
pub fn ln(v: f32) -> f32 {
    // see notes above in `floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).ln() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::logf32(v) };
}

#[inline]
pub fn log2(v: f32) -> f32 {
    #[cfg(target_os = "android")]
    return ::sys::android::log2f32(v);
    #[cfg(not(target_os = "android"))]
    return unsafe { intrinsics::log2f32(v) };
}

#[inline]
pub fn log10(v: f32) -> f32 {
    // see notes above in `floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).log10() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::log10f32(v) };
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
    // see notes in `core::f32::Float::floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).sin() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::sinf32(v) };
}

#[inline]
pub fn cos(v: f32) -> f32 {
    // see notes in `core::f32::Float::floor`
    #[cfg(target_env = "msvc")]
    return (v as f64).cos() as f32;
    #[cfg(not(target_env = "msvc"))]
    return unsafe { intrinsics::cosf32(v) };
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

        #[cfg_attr(all(windows, target_env = "msvc"), link_name = "_hypotf")]
        pub fn hypotf(x: c_float, y: c_float) -> c_float;
    }

    // See the comments in the `floor` function for why MSVC is special
    // here.
    #[cfg(not(target_env = "msvc"))]
    extern {
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
    }

    #[cfg(target_env = "msvc")]
    pub use self::shims::*;
    #[cfg(target_env = "msvc")]
    mod shims {
        use libc::{c_float, c_int};

        #[inline]
        pub unsafe fn acosf(n: c_float) -> c_float {
            f64::acos(n as f64) as c_float
        }

        #[inline]
        pub unsafe fn asinf(n: c_float) -> c_float {
            f64::asin(n as f64) as c_float
        }

        #[inline]
        pub unsafe fn atan2f(n: c_float, b: c_float) -> c_float {
            f64::atan2(n as f64, b as f64) as c_float
        }

        #[inline]
        pub unsafe fn atanf(n: c_float) -> c_float {
            f64::atan(n as f64) as c_float
        }

        #[inline]
        pub unsafe fn coshf(n: c_float) -> c_float {
            f64::cosh(n as f64) as c_float
        }

        #[inline]
        #[allow(deprecated)]
        pub unsafe fn frexpf(x: c_float, value: &mut c_int) -> c_float {
            let (a, b) = f64::frexp(x as f64);
            *value = b as c_int;
            a as c_float
        }

        #[inline]
        #[allow(deprecated)]
        pub unsafe fn ldexpf(x: c_float, n: c_int) -> c_float {
            f64::ldexp(x as f64, n as isize) as c_float
        }

        #[inline]
        pub unsafe fn sinhf(n: c_float) -> c_float {
            f64::sinh(n as f64) as c_float
        }

        #[inline]
        pub unsafe fn tanf(n: c_float) -> c_float {
            f64::tan(n as f64) as c_float
        }

        #[inline]
        pub unsafe fn tanhf(n: c_float) -> c_float {
            f64::tanh(n as f64) as c_float
        }
    }
}
