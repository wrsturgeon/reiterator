/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::arithmetic_side_effects, clippy::integer_arithmetic)]

#[allow(clippy::wildcard_imports)]
use crate::*;
use ::alloc::vec::Vec;

quickcheck::quickcheck! {

    fn never_panics(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        let mut iter = Reiterator::new(v);
        for i in indices {
            assert_eq!(iter.at(i).map(index), (i < size).then_some(i));
        }
        true
    }

    fn always_some_in_bounds(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        if size > 0 {
            let mut iter = Reiterator::new(v);
            for i in indices {
                assert_eq!(iter.at(i % size).map(index), Some(i % size));
            }
        }
        true
    }

    fn always_none_out_of_bounds(v: Vec<bool>, indices: Vec<usize>) -> bool {
        let size = v.len();
        let mut iter = Reiterator::new(v);
        for i in indices {
            if i >= size {
                assert!(iter.at(i).is_none());
            }
        }
        true
    }

    fn correct_range(size: u16, indices: Vec<usize>) -> bool {
        if size > 0 {
            let mut iter = Reiterator::new(0..size);
            for i in indices {
                let indexed = iter.at(i);
                assert_eq!(indexed.map(|x| x.index), (i < usize::from(size)).then_some(i));
                assert_eq!(indexed.map(|x| x.index), indexed.map(|x| usize::from(*x.value)));
            }
        }
        true
    }

}
