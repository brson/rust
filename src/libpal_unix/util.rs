// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use errno::errno;
use libc;
use pal_common::io_error;

pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> io_error::Result<T> {
    if t.is_minus_one() {
        Err(io_error::Error::from_raw_os_error(errno()))
    } else {
        Ok(t)
    }
}

pub fn cvt_r<T, F>(mut f: F) -> io_error::Result<T>
    where T: IsMinusOne,
          F: FnMut() -> T
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => {}
            other => return other,
        }
    }
}
