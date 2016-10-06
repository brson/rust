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
use pal_common::io_error;

pub trait IsZero {
    fn is_zero(&self) -> bool;
}

macro_rules! impl_is_zero {
    ($($t:ident)*) => ($(impl IsZero for $t {
        fn is_zero(&self) -> bool {
            *self == 0
        }
    })*)
}

impl_is_zero! { i8 i16 i32 i64 isize u8 u16 u32 u64 usize }

pub fn cvt<I: IsZero>(i: I) -> io_error::Result<I> {
    if i.is_zero() {
        Err(io_error::Error::from_raw_os_error(errno()))
    } else {
        Ok(i)
    }
}

