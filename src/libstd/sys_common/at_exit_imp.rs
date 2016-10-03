// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of running at_exit routines
//!
//! Documentation can be found on the `rt::at_exit` function.

use alloc::boxed::FnBox;
use ptr;
use sys::mutex::Mutex;

/// Enqueues a procedure to run when the main thread exits.
///
/// Currently these closures are only run once the main *Rust* thread exits.
/// Once the `at_exit` handlers begin running, more may be enqueued, but not
/// infinitely so. Eventually a handler registration will be forced to fail.
///
/// Returns `Ok` if the handler was successfully registered, meaning that the
/// closure will be run once the main thread exits. Returns `Err` to indicate
/// that the closure could not be registered, meaning that it is not scheduled
/// to be run.
pub fn at_exit<F: FnOnce() + Send + 'static>(f: F) -> Result<(), ()> {
    unsafe { AT_EXIT.at_exit(f) }
}

pub fn cleanup() {
    unsafe { AT_EXIT.cleanup() }
}

static mut AT_EXIT: AtExit = AtExit::new();

// The maximum number of times the cleanup routines will be run. While running
// the at_exit closures new ones may be registered, and this count is the number
// of times the new closures will be allowed to register successfully. After
// this number of iterations all new registrations will return `false`.
const ITERS: usize = 10;

type Queue = Vec<Box<FnBox()>>;

// NB these are specifically not types from `std::sync` as they currently rely
// on poisoning and this module needs to operate at a lower level than requiring
// the thread infrastructure to be in place (useful on the borders of
// initialization/destruction).
struct AtExit {
    lock: Mutex,
    queue: *mut Queue
}

impl AtExit {
    pub const fn new() -> AtExit {
        AtExit {
            lock: Mutex::new(),
            queue: ptr::null_mut()
        }
    }

    pub fn at_exit<F: FnOnce() + Send + 'static>(&mut self, f: F) -> Result<(), ()> {
        if self.push(Box::new(f)) {Ok(())} else {Err(())}
    }

    fn push(&mut self, f: Box<FnBox()>) -> bool {
        let mut ret = true;
        unsafe {
            self.lock.lock();
            if self.init() {
                (*self.queue).push(f);
            } else {
                ret = false;
            }
            self.lock.unlock();
        }
        ret
    }

    unsafe fn init(&mut self) -> bool {
        if self.queue.is_null() {
            let state: Box<Queue> = box Vec::new();
            self.queue = Box::into_raw(state);
        } else if self.queue as usize == 1 {
            // can't re-init after a cleanup
            return false
        }

        true
    }

    pub fn cleanup(&mut self) {
        for i in 0..ITERS {
            unsafe {
                self.lock.lock();
                let queue = self.queue;
                self.queue = if i == ITERS - 1 {1} else {0} as *mut _;
                self.lock.unlock();

                // make sure we're not recursively cleaning up
                assert!(queue as usize != 1);

                // If we never called init, not need to cleanup!
                if queue as usize != 0 {
                    let queue: Box<Queue> = Box::from_raw(queue);
                    for to_run in *queue {
                        to_run();
                    }
                }
            }
        }
    }
}

