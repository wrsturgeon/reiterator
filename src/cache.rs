/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Cache that only works with iterator-like structures.
//! This file shouldn't have a single instace of the term `mut` (other than this one lol).

#![allow(box_pointers)]

use ::alloc::{boxed::Box, vec, vec::Vec};
use ::core::{marker::PhantomData, pin::Pin};

/// Cache that works with iterator-like structures.
/// Note that all operations are `const` since there are no user-facing mutations.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Cache<'cache, I: Iterator>
where
    Self: 'cache, // Necessary to make sure `'cache` is _exactly_ this struct's lifetime, not longer!
    I: 'cache,
    I::Item: 'cache,
{
    /// Iterator producing the input being cached.
    iter: I,
    /// Vector of cached inputs.
    vec: Vec<Pin<Box<I::Item>>>, // TODO: vector of buffers
    /// Lifetime of this `struct`.
    lifetime: PhantomData<&'cache ::core::convert::Infallible>,
    /// Record of the first memory location for each index.
    #[cfg(test)]
    record: Vec<*const I::Item>,
}

impl<'cache, I: Iterator> Cache<'cache, I> {
    /// Initialize a new empty cache.
    #[inline(always)]
    pub fn new<II: IntoIterator<IntoIter = I>>(into_iter: II) -> Self {
        Self {
            iter: into_iter.into_iter(),
            vec: vec![],
            lifetime: PhantomData,
            #[cfg(test)]
            record: vec![],
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
    pub fn get(&mut self, index: usize) -> Option<&'cache I::Item> {
        loop {
            if let Some(cached) = self.vec.get(index) {
                let pointer: *const _ = cached.as_ref().get_ref();
                // SAFETY:
                // Removing `mut` from the lifetime but otherwise leaving it untouched.
                // Since it's `Pin`ned and has the exact same lifetime as this borrow,
                // Rust statically asserts that we don't invalidate it later.
                #[allow(unsafe_code)]
                return Some(unsafe { &*pointer });
            }
            let pinned_memory_location = Box::pin(self.iter.next()?);
            #[cfg(test)]
            {
                self.record.push(pinned_memory_location.as_ref().get_ref());
            }
            self.vec.push(pinned_memory_location);
            #[cfg(test)]
            {
                assert_eq!(self.vec.len(), self.record.len());
                for (i, (v, &r)) in self.vec.iter().zip(self.record.iter()).enumerate() {
                    let vref = v.as_ref().get_ref();
                    assert!(::core::ptr::eq(vref, r), "Element #{i:} corrupt");
                }
            }
        }
    }
}

/// Create a `Cache` from anything that can be turned into an `Iterator`.
#[inline(always)]
#[must_use]
pub fn cached<'cache, I: IntoIterator>(iter: I) -> Cache<'cache, I::IntoIter> {
    Cache::new(iter)
}

/// Pipe the output of an `IntoIterator` to make a `Reiterator`.
pub trait Cached: IntoIterator {
    /// Create a `Reiterator` from anything that can be turned into an `Iterator`.
    #[must_use]
    fn cached<'cache>(self) -> Cache<'cache, Self::IntoIter>;
}

impl<I: IntoIterator> Cached for I {
    #[inline(always)]
    #[must_use]
    fn cached<'cache>(self) -> Cache<'cache, Self::IntoIter> {
        cached(self)
    }
}
