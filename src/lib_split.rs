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
//! # fn main() { opt().unwrap(); } fn opt() -> Option<()> {
//! use reiterator::{Reiterate, value};
//! let mut iter = vec!['a', 'b', 'c'].reiterate()?; // None of the values are computed or cached at construction.
//! iter.get(1); // Only now computes the values, and computes only the first two.
//! assert_eq!(iter.read().map(value), Some(&'b')); // Reference to the cached value.
//! # Some(()) }
//! ```
//!
//! You can drive it like a normal `Iterator`:
//!
//! ```rust
//! # fn main() { opt().unwrap(); } fn opt() -> Option<()> {
//! use reiterator::{Indexed, Reiterate, value};
//! let mut iter = vec!['a', 'b', 'c'].reiterate()?; // Computes the first value right here, returns `None` if empty.
//! let first_value = iter.read();
//! assert_eq!(first_value, Some(Indexed { index: 0, value: &'a' }));
//! assert_eq!(iter.read(), Some(Indexed { index: 0, value: &'a' })); // Exact same reference to the same `char` in memory
//! iter.next(); // Computes but does not return; returning a lifetime from an `&mut self` function is generally a nightmare
//! assert_eq!(iter.read(), Some(Indexed { index: 1, value: &'b' }));
//! iter.next();
//! assert_eq!(iter.read(), Some(Indexed { index: 2, value: &'c' }));
//! iter.next();
//! assert_eq!(iter.read(), None);
//!
//! // But then you can rewind and do it all again for free, returning cached references to the same values we just made:
//! iter.restart(); // Equivalent to `iter.get(0)`
//! assert_eq!(iter.read(), Some(Indexed { index: 0, value: &'a' }));
//! iter.next();
//! assert_eq!(iter.read(), Some(Indexed { index: 1, value: &'b' }));
//! iter.next();
//! assert_eq!(iter.read(), Some(Indexed { index: 2, value: &'c' }));
//! iter.next();
//! assert_eq!(iter.read(), None);
//!
//! // Or start from anywhere:
//! iter.get(1);
//! assert_eq!(iter.read(), Some(Indexed { index: 1, value: &'b' }));
//! iter.next();
//! assert_eq!(iter.read(), Some(Indexed { index: 2, value: &'c' }));
//! iter.next();
//! assert_eq!(iter.read(), None);
//!
//! // And just to prove that it's literally a reference to the same cached value in memory:
//! iter.restart();
//! assert!(std::ptr::eq(first_value.unwrap().value, iter.read().unwrap().value));
//! # Some(()) }
//! ```

#![no_std]
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
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::inline_always,
    clippy::match_ref_pats,
    clippy::mod_module_files,
    clippy::question_mark_used,
    clippy::separated_literal_suffix
)]

extern crate alloc;

#[cfg(test)]
mod test;

/// A value as well as how many elements an iterator spat out before it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)]
pub struct Indexed<'a, A> {
    /// Number of elements an iterator spat out before this one.
    pub index: usize,

    /// Output of an iterator.
    pub value: &'a A,
}

/// Return the index from an `Indexed` item. Consumes its argument: written with `.map(index)` in mind.
#[allow(clippy::needless_pass_by_value)]
#[inline(always)]
#[must_use]
pub const fn index<A>(indexed: Indexed<'_, A>) -> usize {
    indexed.index
}

/// Return the value from an `Indexed` item. Consumes its argument: written with `.map(value)` in mind.
#[allow(clippy::needless_pass_by_value)]
#[inline(always)]
#[must_use]
pub const fn value<A>(indexed: Indexed<'_, A>) -> &A {
    indexed.value
}

/// Caching repeatable iterator that only ever calculates each element once.
/// NOTE that if the iterator is not referentially transparent (i.e. pure, e.g. mutable state), this *will not necessarily work*!
/// We replace a call to a previously evaluated index with the value we already made, so side effects will not show up at all.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Reiterator<I: Iterator> {
    /// Iterator that this struct thinly wraps.
    iter: I,

    /// Store of previously computed (referentially transparent) values.
    cache: ::alloc::vec::Vec<I::Item>,

    /// Index of the item this iterator _will_ return when we call `get`.
    /// Mutable: you can assign _any_ value, even out of bounds, and nothing will break:
    ///   - If the index is in bounds, the next time you call `get`, we calculate each element until this one (if not already cached).
    ///   - If the index is out of bounds, we return `None` (after exhausting the iterator, though: it's not necessarily a fixed size).
    /// Note that this doesn't mean that this index's value has been calculated yet.
    index: usize,

    #[cfg(debug_assertions)]
    populated: bool,
}

impl<I: Iterator> Reiterator<I> {
    /// EFFECTIVE RULE:
    /// Anytime we have an `&mut self` function, **the return type cannot have a lifetime**.
    /// For clarity for end-users, these should have no return type at all (i.e. `()`).

    /// Read the first element from the iterator.
    /// If it exists, set up this iterator to `read` from it.
    /// Otherwise, return None.
    #[inline(always)]
    #[must_use]
    pub fn new<II: IntoIterator<IntoIter = I>>(iter: II) -> Option<Self> {
        let mut iter = iter.into_iter();
        let cache = ::alloc::vec![iter.next()?];
        Some(Self {
            iter,
            cache,
            index: 0,
            #[cfg(debug_assertions)]
            populated: true,
        })
    }

    /// Set the index to zero. Literal drop-in equivalent for `.index = 0`, always inlined. Clearer, I guess.
    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Set the index and calculate items up to and including it.
    #[inline(always)]
    pub fn get(&mut self, index: usize) {
        self.index = index;
        self.populate();
    }

    /// Set the index to zero. Literal drop-in equivalent for `.get(0)`, always inlined. Clearer, I guess.
    #[inline(always)]
    pub fn restart(&mut self) {
        self.get(0);
    }

    /// Return the element at the requested index if and only if we have already calculated it.
    /// NOTE that failure here does not necessarily mean the value is out of bounds;
    /// it simply means that we haven't computed it yet, and it _might_ be out of bounds.
    #[inline]
    #[must_use]
    pub fn read_index(&self, index: usize) -> Option<Indexed<'_, I::Item>> {
        debug_assert!(self.populated);
        self.cache.get(index).map(|value| Indexed { index, value })
    }

    /// Return the element at the current index if and only if we have already calculated it.
    /// NOTE that failure here does not necessarily mean the value is out of bounds;
    /// it simply means that we haven't computed it yet, and it _might_ be out of bounds.
    #[inline(always)]
    #[must_use]
    pub fn read(&self) -> Option<Indexed<'_, I::Item>> {
        self.read_index(self.index)
    }

    /// Compute up to and including the current index if not already cached.
    #[inline(always)]
    pub fn populate(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.populated = true;
        }
        while self.cache.get(self.index).is_none() {
            if let Some(item) = self.iter.next() {
                self.cache.push(item);
            } else {
                return;
            }
        }
    }

    /// Advance the index and compute up to and including the corresponding value if not already cached.
    #[inline(always)]
    pub fn next(&mut self) {
        #![allow(clippy::arithmetic_side_effects, clippy::integer_arithmetic)]
        self.get(self.index + 1);
    }
}

/// Create a `Reiterator` from anything that can be turned into an `Iterator`.
#[inline(always)]
#[must_use]
pub fn reiterate<I: IntoIterator>(iter: I) -> Option<Reiterator<I::IntoIter>> {
    Reiterator::new(iter)
}

/// Pipe the output of an `IntoIter` to make a `Reiterator`.
pub trait Reiterate: IntoIterator {
    /// Create a `Reiterator` from anything that can be turned into an `Iterator`.
    #[must_use]
    fn reiterate(self) -> Option<Reiterator<Self::IntoIter>>;
}

impl<I: IntoIterator> Reiterate for I {
    #[inline(always)]
    #[must_use]
    fn reiterate(self) -> Option<Reiterator<Self::IntoIter>> {
        reiterate(self)
    }
}
