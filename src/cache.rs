/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Cache that only works with iterator-like structures.
//! This file shouldn't have a single instace of the term `mut` (other than this one lol).

#![allow(box_pointers)]

use ::alloc::{vec, vec::Vec};

/// Cache that works with iterator-like structures.
/// Note that all operations are `const` since there are no user-facing mutations.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cache<I: Iterator> {
    /// Iterator producing the input being cached.
    iter: I,
    /// Vector of cached inputs.
    vec: Vec<I::Item>,
}

impl<I: Iterator> Cache<I> {
    /// Initialize a new empty cache.
    #[inline(always)]
    pub fn new<II: IntoIterator<IntoIter = I>>(into_iter: II) -> Self {
        Self {
            iter: into_iter.into_iter(),
            vec: vec![],
        }
    }

    /// Whether this cache holds any cached elements.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// If not already cached, repeatedly call `next` until we either reach `index` or `next` returns `None`.
    /// Immutably borrow this entire `Cache` for the duration of your returned reference.
    #[inline]
    pub fn get(&mut self, index: usize) -> Option<&I::Item> {
        loop {
            if let cached @ Some(_) = {
                let v: *const _ = &self.vec;
                #[allow(unsafe_code)]
                unsafe { &*v }.get(index)
            } {
                return cached;
            }
            self.vec.push(self.iter.next()?);
        }
    }
}

/// Create a `Cache` from anything that can be turned into an `Iterator`.
#[inline(always)]
#[must_use]
pub fn cached<I: IntoIterator>(iter: I) -> Cache<I::IntoIter> {
    Cache::new(iter)
}

/// Pipe the output of an `IntoIterator` to make a `Reiterator`.
pub trait Cached: IntoIterator {
    /// Create a `Reiterator` from anything that can be turned into an `Iterator`.
    #[must_use]
    fn cached(self) -> Cache<Self::IntoIter>;
}

impl<I: IntoIterator> Cached for I {
    #[inline(always)]
    #[must_use]
    fn cached(self) -> Cache<Self::IntoIter> {
        cached(self)
    }
}
