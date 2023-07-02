/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Cache that only works with iterator-like structures.
//! This file shouldn't have a single instace of the term `mut` (other than this one lol).

#![allow(box_pointers)]

use ::alloc::{vec, vec::Vec};
use ::core::{cell::RefCell, pin::Pin};
use alloc::boxed::Box;

/// Cache that only works with iterator-like structures.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Cache<I: Iterator> {
    /// Iterator producing the input being cached.
    iter: RefCell<I>,
    /// Vector of cached inputs.
    vec: RefCell<Vec<Pin<Box<I::Item>>>>, // TODO: vector of buffers
}

impl<I: Iterator> Cache<I> {
    /// Initialize a new empty cache.
    #[inline(always)]
    pub const fn new(i: I) -> Self {
        Self {
            iter: RefCell::new(i),
            vec: RefCell::new(vec![]),
        }
    }

    /// If not already cached, repeatedly call `next` until we either reach `index` or `next` returns `None`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&I::Item> {
        loop {
            if let Some(cached) = self.vec.borrow().get(index) {
                return Some(
                    #[allow(clippy::as_conversions, trivial_casts, unsafe_code)]
                    // SAFETY: Pinned addresses with the correct lifetime. Property-tested as well.
                    unsafe {
                        &*(cached.as_ref().get_ref() as *const _)
                    },
                );
            }
            self.vec
                .borrow_mut()
                .push(Box::pin(self.iter.borrow_mut().next()?));
        }
    }
}
