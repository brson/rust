// Copyright 2012-2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// FIXME: This module should not be necessary. The pal_* crates are supposed to
// be all different implementations of a crate named 'pal', built by Cargo as
// 'pal', but today I don't know how to do that, so this module maps the
// platform-specific crate names to 'pal'.

#[cfg(unix)]
extern crate pal_unix as imp;

#[cfg(windows)]
extern crate pal_windows as imp;

pub use self::imp::*;
