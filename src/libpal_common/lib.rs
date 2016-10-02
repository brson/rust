// Copyright 2012-2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Platform abstraction support

#![crate_name = "pal_common"]
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
#![feature(collections)]
#![feature(fused)]
#![feature(libc)] // temporary hack for c_str
#![feature(int_error_internals)]
#![feature(question_mark)]
#![feature(reflect_marker)]
#![feature(staged_api)]
#![feature(try_borrow)]
#![feature(try_from)]
#![feature(unicode)]

extern crate alloc;
extern crate collections;
extern crate rustc_unicode;

pub mod ascii;
pub mod c_str;
pub mod error;
pub mod memchr;
