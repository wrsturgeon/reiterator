/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! # Reiterator
//!
//! You know that one friend who can tell the same story over and over again, but you only really listen the first time? Maybe they even have a _set_ of stories so interesting you only really needed to listen to it once?
//!
//! This crate is that friend.
//!
//! ## What?
//!
//! This `no_std` crate takes an iterator and caches its output, allowing you to access an immutable reference to the output of any referentially transparent iterator call.
//! Rewind it or set it ten elements ahead, and it'll gladly oblige, but only when you ask it. A little taste of lazy evaluation in eager-flavored Rust.
//! Plus, it returns the index of the value as well, but there are built-in `map`-compatible mini-functions to get either the value or the index only:
//!
//! ```rust
//! use reiterator::{OptionIndexed, Reiterate, value};
//! let iter = vec!['a', 'b', 'c'].reiterate(); // None of the values are computed until...
//! let indexed = iter.at(1); // here. We only compute the first two, and we cache their results.
//! assert!(indexed.is_some());
//! assert_eq!(indexed.value(), Some(&'b'));
//! assert_eq!(indexed.index(), Some(1));
//! let _ = iter.at(2); // Calls the iterator only once
//! let _ = iter.at(0); let _ = iter.at(1); let _ = iter.at(2); // All cached! Just a few clocks and pulling from the heap.
//! ```
//!
//! You can drive it like a normal `Iterator`:
//!
//! ```rust
//! use reiterator::{Indexed, Reiterate, value};
//! let mut iter = vec!['a', 'b', 'c'].reiterate(); // None of the values are computed or cached until...
//! assert_eq!(iter.get(), Some(Indexed { index: 0, value: &'a' })); // here.
//! assert_eq!(iter.get(), Some(Indexed { index: 0, value: &'a' })); // Using the cached version
//! assert_eq!(iter.next(), Some(1)); // Note that `next` doesn't return a value for simplicity: would it return 'a' or 'b'?
//! assert_eq!(iter.get(), Some(Indexed { index: 1, value: &'b' })); // ...but it does change the internal index
//! assert_eq!(iter.get(), Some(Indexed { index: 1, value: &'b' }));
//! assert_eq!(iter.next(), Some(2));
//! assert_eq!(iter.get(), Some(Indexed { index: 2, value: &'c' }));
//! assert_eq!(iter.next(), None); // Off the end of the iterator!
//! assert_eq!(iter.get(), None);
//!
//! // But then you can rewind and do it all again for free, returning cached references to the same values we just made:
//! iter.restart();
//! assert_eq!(iter.get(), Some(Indexed { index: 0, value: &'a' }));
//! assert_eq!(iter.next(), Some(1));
//! assert_eq!(iter.get(), Some(Indexed { index: 1, value: &'b' }));
//!
//! // Or start from anywhere:
//! iter.index.set(1);
//! assert_eq!(iter.get(), Some(Indexed { index: 1, value: &'b' }));
//! assert_eq!(iter.next(), Some(2));
//! assert_eq!(iter.get(), Some(Indexed { index: 2, value: &'c' }));
//! assert_eq!(iter.at(1), Some(Indexed { index: 1, value: &'b' }));
//! assert_eq!(iter.at(2), Some(Indexed { index: 2, value: &'c' }));
//! assert_eq!(iter.at(3), None);
//!
//! // And just to prove that it's literally a reference to the same cached value in memory:
//! assert!(std::ptr::eq(iter.at(0).unwrap().value, iter.at(0).unwrap().value));
//! ```

#![cfg_attr(not(test), no_std)]
#![deny(warnings)]
#![warn(
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo,
    elided_lifetimes_in_paths,
    missing_docs,
    rustdoc::all
)]
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
#![warn(
    absolute_paths_not_starting_with_crate,
    box_pointers,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_results,
    unused_tuple_struct_fields,
    variant_size_differences
)]
#![allow(
    box_pointers,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::inline_always,
    clippy::match_ref_pats,
    clippy::mod_module_files,
    clippy::pub_use,
    clippy::question_mark_used,
    clippy::separated_literal_suffix,
    clippy::single_char_lifetime_names
)]

extern crate alloc;

pub mod cache;
pub mod indexed;

#[cfg(test)]
mod test;

use cache::{Cache, Cached};
use indexed::Indexed;

/// Caching repeatable iterator that only ever calculates each element once.
/// NOTE that if the iterator is not referentially transparent (i.e. pure, e.g. mutable state), this *will not necessarily work*!
/// We replace a call to a previously evaluated index with the value we already made, so side effects will not show up at all.
#[allow(missing_debug_implementations, clippy::partial_pub_fields)]
pub struct Reiterator<'cache, I: Iterator>
where
    Self: 'cache,
    I::Item: 'cache,
{
    /// Iterator and a store of previously computed (referentially transparent) values.
    cache: Cache<'cache, I>,

    /// Safe to edit! Assign _any_ value, even out of bounds, and nothing will break:
    ///   - If the index is in bounds, the next time you call `get`/`next`, we calculate each element until this one (if not already cached).
    ///   - If the index is out of bounds, we return `None` (after exhausting the iterator: it's not necessarily a fixed size, so there's only one way to find out).
    /// Note that this iterator is lazy, so assigning an index doesn't mean that the value at that index has been calculated.
    pub index: usize,
}

impl<'cache, I: Iterator> Reiterator<'cache, I> {
    /// Set up the iterator to return the first element, but don't calculate it yet.
    #[inline(always)]
    #[must_use]
    pub fn new(cache: Cache<'cache, I>) -> Self {
        Self { cache, index: 0 }
    }

    /// Set the index to zero. Literal drop-in equivalent for `.index = 0`, always inlined. Clearer, I guess.
    #[inline(always)]
    pub fn restart(&mut self) {
        self.index = 0;
    }

    /// Return the element at the requested index *or compute it if we haven't*, provided it's in bounds.
    #[inline]
    #[must_use]
    pub fn at(&mut self, index: usize) -> Option<&'cache I::Item> {
        self.cache.get(index).map(|item| {
            let pointer: *const _ = item;
            #[allow(unsafe_code)]
            // SAFETY: Known lifetime.
            unsafe {
                &*pointer
            }
        })
    }

    /// Return the current element or compute it if we haven't, provided it's in bounds.
    /// This can be called any number of times in a row to return the exact same item;
    /// we won't advance to the next element until you explicitly call `next`.
    #[inline(always)]
    #[must_use]
    pub fn get(&mut self) -> Option<Indexed<'cache, I::Item>> {
        self.at(self.index).map(|value| Indexed {
            index: self.index,
            value,
        })
    }

    /// Advance the index without computing the corresponding value.
    #[inline(always)]
    pub fn lazy_next(&mut self) -> Option<usize> {
        self.index.checked_add(1).map(|incr| {
            self.index = incr;
            incr
        })
    }
}

impl<'cache, I: Iterator> Iterator for Reiterator<'cache, I> {
    type Item = Indexed<'cache, I::Item>;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let tmp = self.get();
        let _ = self.lazy_next()?;
        tmp
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for Reiterator<'_, I> {}

/// Create a `Reiterator` from anything that can be turned into an `Iterator`.
#[inline(always)]
#[must_use]
pub fn reiterate<'cache, I: IntoIterator>(iter: I) -> Reiterator<'cache, I::IntoIter> {
    Reiterator {
        cache: iter.cached(),
        index: 0,
    }
}

/// Pipe the output of an `IntoIter` to make a `Reiterator`.
pub trait Reiterate: IntoIterator {
    /// Create a `Reiterator` from anything that can be turned into an `Iterator`.
    #[must_use]
    fn reiterate<'cache>(self) -> Reiterator<'cache, Self::IntoIter>;
}

impl<I: IntoIterator> Reiterate for I {
    #[inline(always)]
    #[must_use]
    fn reiterate<'cache>(self) -> Reiterator<'cache, Self::IntoIter> {
        reiterate(self)
    }
}
