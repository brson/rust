// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(target_os = "android")]

// Back in the day [1] the `signal` function was just an inline wrapper
// around `bsd_signal`, but starting in API level android-20 the `signal`
// symbols was introduced [2]. Finally, in android-21 the API `bsd_signal` was
// removed [3].
//
// Basically this means that if we want to be binary compatible with multiple
// Android releases (oldest being 9 and newest being 21) then we need to check
// for both symbols and not actually link against either.
//
// [1]: https://chromium.googlesource.com/android_tools/+/20ee6d20/ndk/platforms
//                                       /android-18/arch-arm/usr/include/signal.h
// [2]: https://chromium.googlesource.com/android_tools/+/fbd420/ndk_experimental
//                                       /platforms/android-20/arch-arm
//                                       /usr/include/signal.h
// [3]: https://chromium.googlesource.com/android_tools/+/20ee6d/ndk/platforms
//                                       /android-21/arch-arm/usr/include/signal.h
pub unsafe fn signal(signum: c_int, handler: sighandler_t) -> sighandler_t {
    weak!(fn signal(c_int, sighandler_t) -> sighandler_t);
    weak!(fn bsd_signal(c_int, sighandler_t) -> sighandler_t);

    let f = signal.get().or_else(|| bsd_signal.get());
    let f = f.expect("neither `signal` nor `bsd_signal` symbols found");
    f(signum, handler)
}
