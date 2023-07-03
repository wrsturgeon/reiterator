/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::arithmetic_side_effects, clippy::integer_arithmetic)]

#[allow(clippy::wildcard_imports)]
use ::alloc::{vec, vec::Vec};

use crate::{cache::Cached, indexed::Indexed, Reiterate};

#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
#[test]
fn persistent_addresses_cache() {
    let range = 0..=u16::MAX;
    let mut cache = range.clone().cached();
    let mut addresses = vec![];
    for i in range.clone() {
        addresses.push(cache.get(usize::from(i)).unwrap());
    }
    for i in range {
        assert_eq!(addresses[usize::from(i)], &i);
    }
}

#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
#[test]
fn persistent_addresses_reiterator() {
    let range = 0..=u8::MAX;
    let mut addresses = vec![];
    // Create a *temporary* `Reiterator` here:
    // Rust needs to be able to figure out that
    // it needs to live until the end of the function
    for i in range.clone().reiterate() {
        println!("{i:#?}");
        addresses.push(i);
    }
    assert_eq!(addresses.len(), range.len());
    // Vec just underwent a metric fuckton of reallocations
    // But we hold the original memory locations
    // So this test is crucial
    for (i, addr) in addresses.into_iter().enumerate() {
        println!("   i = {i:}");
        println!("addr = {addr:#?}");
        println!();
        assert_eq!(
            addr,
            Indexed {
                index: i,
                value: &i.try_into().unwrap()
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
    let mut cache = (0..=u16::MAX).cached();
    for i in 0..=u16::MAX {
        let lhs = cache.get(usize::from(i));
        let rhs = Some(&i);
        assert_eq!(lhs, rhs);
    }
}

quickcheck::quickcheck! {

    fn prop_cache_range(indices: ::alloc::vec::Vec<u8>) -> bool {
        let mut cache = (0..=u8::MAX).cached();
        indices.into_iter().all(|i| {
            cache.get(usize::from(i)).is_some_and(|v| v == &i)
        })
    }

    fn prop_always_some_in_bounds(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        if size > 0 {
            let mut iter = v.reiterate();
            for i in indices {
                assert!(iter.at(i % size).is_some());
            }
        }
        true
    }

    fn prop_always_none_out_of_bounds(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        let mut iter = v.reiterate();
        for i in indices {
            if i >= size {
                assert!(iter.at(i).is_none());
            }
        }
        true
    }

    fn prop_correct_range(size: u8, indices: Vec<u8>) -> bool {
        if size > 0 {
            let mut iter = (0..=size).reiterate();
            for i in indices {
                assert_eq!(iter.at(usize::from(i)), (i <= size).then_some(&i));
            }
        }
        true
    }


    fn prop_persistent_addresses_cache(v: Vec<u16>) -> bool {
        let mut cache = (0..=u16::MAX).cached();
        let mut addresses = vec![];
        for i in v.iter() {
            addresses.push(cache.get(usize::from(*i)).unwrap());
        }
        assert_eq!(addresses.len(), v.len());
        // Vec just underwent a metric fuckton of reallocations
        // But we hold the original memory locations
        // So this test is crucial
        addresses.into_iter().zip(v).all(|(a, v)| a == &v)
    }

}
