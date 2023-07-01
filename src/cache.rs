/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Cache that only works with iterator-like structures.
//! This file shouldn't have a single instace of the term `mut` (other than this one lol).

use ::alloc::{vec, vec::Vec};
use ::core::cell::{Ref, RefCell};

/// Cache that only works with iterator-like structures.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Cache<A, F: FnMut() -> Option<A>> {
    vec: RefCell<Vec<A>>,
    f: RefCell<F>,
}

impl<A, F: FnMut() -> Option<A>> Cache<A, F> {
    /// Initialize a new empty cache.
    pub const fn new(f: F) -> Self {
        Self {
            vec: RefCell::new(vec![]),
            f: RefCell::new(f),
        }
    }

    /// If not already cached, repeatedly call `next` until we either reach `index` or `next` returns `None`.
    pub fn get(&self, index: usize) -> Option<Ref<'_, A>> {
        loop {
            if let cached @ Some(_) = Ref::filter_map(self.vec.borrow(), |v| v.get(index)).ok() {
                return cached;
            } else {
                self.vec.borrow_mut().push(self.f.borrow_mut()()?);
            }
        }
    }
}
