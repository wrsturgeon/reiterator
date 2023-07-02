/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::arithmetic_side_effects, clippy::integer_arithmetic)]

#[allow(clippy::wildcard_imports)]
use crate::*;
use ::alloc::{vec, vec::Vec};

#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
#[test]
fn persistent_addresses_cache() {
    let range = 0..u16::MAX;
    let cache = Cache::new(range.clone());
    let mut addresses = vec![];
    for i in range.clone() {
        addresses.push(cache.get(usize::from(i)).unwrap());
    }
    for i in range {
        assert_eq!(*addresses[usize::from(i)], i);
    }
}

#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
#[test]
fn persistent_addresses_reiterator() {
    let range = 0..u16::MAX;
    let iter = range.clone().reiterate();
    let mut addresses = vec![];
    loop {
        addresses.push(iter.get().unwrap());
        if iter.next().is_none() {
            break;
        }
    }
    for i in range {
        assert_eq!(
            addresses[usize::from(i)],
            Indexed {
                index: usize::from(i),
                value: &i
            }
        );
    }
}

/// Test vector reallocation.
/// Vectors are usually implemented as vectors that occasionally double their size,
/// and if you can't double it in place (e.g. if someone else owns the memory just to your right),
/// it'll copy all the elements to wherever you can buy a plot of land twice the current size.
/// In this case, all references are immediately invalidated.
/// (This verifiably happens with a usual `Vec<A>`.)
/// Experimenting with `Pin`s and two layers of indirection.
#[test]
fn simple_range_doesnt_panic() {
    let cache = Cache::new(0..u16::MAX);
    for i in 0..u16::MAX {
        let lhs = cache.get(usize::from(i));
        let rhs = Some(&i);
        assert_eq!(lhs, rhs);
    }
}

quickcheck::quickcheck! {

    fn cache_range(indices: ::alloc::vec::Vec<u8>) -> bool {
        let cache = Cache::new(0..=u8::MAX);
        indices.into_iter().all(|i| {
            cache.get(usize::from(i)).is_some_and(|v| *v == i)
        })
    }

    fn never_panics(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        let iter = Reiterator::new(v);
        for i in indices {
            assert_eq!(iter.at(i).map(index), (i < size).then_some(i));
        }
        true
    }

    fn always_some_in_bounds(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        if size > 0 {
            let iter = Reiterator::new(v);
            for i in indices {
                assert_eq!(iter.at(i % size).map(index), Some(i % size));
            }
        }
        true
    }

    fn always_none_out_of_bounds(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        let iter = Reiterator::new(v);
        for i in indices {
            if i >= size {
                assert!(iter.at(i).is_none());
            }
        }
        true
    }

    fn correct_range(size: u8, indices: Vec<usize>) -> bool {
        if size > 0 {
            let iter = Reiterator::new(0..size);
            for i in indices {
                let indexed = iter.at(i);
                assert_eq!(indexed.as_ref().map(|x| x.index), (i < usize::from(size)).then_some(i));
                assert_eq!(indexed.as_ref().map(|x| x.index), indexed.map(|x| usize::from(*x.value)));
            }
        }
        true
    }

}
