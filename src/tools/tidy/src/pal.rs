// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tidy check to enforce rules about platform-specific code in std
//!
//! This is intended to maintain existing standards of code
//! organization in hopes that std will continue to be refactored to
//! isolate platform-specific bits, making porting easier.
//!
//! This generally means placing restrictions on where `cfg(unix)`,
//! `cfg(windows)`, `cfg(target_os)` and `cfg(target_env)` may appear.
//!
//! The general objective is to isolate platform-specific code to the
//! platform-specific `std::sys` modules, and to the allocation and
//! unwinding crates.
//!
//! Following are the basic rules, though there are currently
//! exceptions:
//!
//! - core may not have platform-specific code
//! - liballoc_system may have platform-specific code
//! - liballoc_jemalloc may have platform-specific code
//! - libpanic_abort may have platform-specific code
//! - libpanic_unwind may have platform-specific code
//! - other crates in the std facade may not
//! - std may have platform-specific code in the following places
//!   - rtdeps.rs - until it is obsoleted by rustbuild
//!   - sys/unix/
//!   - sys/windows/
//!   - os/
//!   - num/{f32.rs, f64.rs} - not sure what to do about these yet
//!
//! Finally, because std contains tests with platform-specific
//! `ignore` attributes, once the parser encounters `mod tests`,
//! platform-specific cfgs are allowed. Not sure yet how to deal with
//! this in the long term.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::iter::Iterator;

// Crates that are part of the standard library
const STD_CRATES: &'static [&'static str] = &[
    "libcore",
    "liballoc",
    "liballoc_jemalloc",
    "liballoc_system",
    "libcollections",
    "liblibc",
    "libpanic_abort",
    "libpanic_unwind",
    "librand",
    "libstd",
    "libtest",
];

// Paths that may contain platform-specific code
const EXCEPTION_PATHS: &'static [&'static str] = &[
    "liballoc_jemalloc",
    "liballoc_system",
    "liblibc",
    "libpanic_abort",
    "libpanic_unwind",
    "libstd/sys",
    "libstd/rtdeps.rs",
    "libstd/os",
    "libstd/num/f32.rs",
    "libstd/num/f64.rs",
];

// Paths that may not contain platform-specific code.
// Basically just used to un-whitelist sys/common.
const RESTRICTED_PATHS: &'static [&'static str] = &[
    "libstd/sys/common"
];

pub fn check(path: &Path, bad: &mut bool) {
    let ref mut contents = String::new();
    super::walk(path, &mut super::filter_dirs, &mut |file| {
        let filename = file.file_name().unwrap().to_string_lossy();
        let is_std_crate = STD_CRATES.iter().any(|s| filename.contains(s));

        if !is_std_crate { return }

        let is_exception_path = EXCEPTION_PATHS.iter().any(|s| filename.contains(s));
        let is_restricted_path = RESTRICTED_PATHS.iter().any(|s| filename.contains(s));

        let do_cfg_check = match (is_exception_path,
                                is_restricted_path) {
            (_, true) => true,
            (true, _) => false,
            _         => true,
        };

        if !do_cfg_check { return }

        check_cfgs(contents, &file);
    });
}

fn check_cfgs(contents: &mut String, file: &Path) {
    t!(t!(File::open(file), file).read_to_string(contents));

    // For now it's ok to have platform-specific code after 'mod tests'.
    let mod_tests_idx = contents.find("mod tests").unwrap_or(contents.len());
    let contents = &contents[..mod_tests_idx];
    let cfgs = parse_cfgs(contents);
}

fn parse_cfgs(contents: &str) -> impl Iterator<Item = (usize, &str)> {
    let candidate_cfgs = contents.match_indices("cfg");
    let candidate_cfg_idxs = candidate_cfgs.map(|(i, _)| i);
    // This is puling out the indexes of all "cfg" strings
    // that appear to be tokens succeeded by a paren.
    let cfgs = candidate_cfg_idxs.map(|i| {
        let pre_idx = i.saturating_sub(i);
        let succeeds_non_ident = !contents.get(pre_idx)
            .map(char::from)
            .map(char::is_alphanumeric)
            .unwrap_or(false);
        let contents_after = contents[i + 1..];
        let first_paren = contents_after.find('(');
        let paren_idx = first_paren.map(|ip| i + 1 + ip);
        let preceeds_whitespace_and_paren = paren_idx.map(|ip| {
            let maybe_space = contents[i + 1 .. ip];
            maybe_space.is_whitespace()
        }).unwrap_or(false);
    });
}
