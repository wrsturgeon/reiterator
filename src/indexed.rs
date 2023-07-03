/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Struct holding an index, a reference to a value, _and a lifetimed reference to the vector that holds the value_.

/// A value as well as how many elements an iterator spat out before it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)]
pub struct Indexed<'value, Value> {
    /// Number of elements an iterator spat out before this one.
    pub index: usize,

    /// Output of an iterator.
    pub value: &'value Value,
}

/// Return the index from an `Indexed` item. Consumes its argument: written with `.map(index)` in mind.
#[allow(clippy::needless_pass_by_value)]
#[inline(always)]
#[must_use]
pub const fn index<Value>(indexed: Indexed<'_, Value>) -> usize {
    indexed.index
}

/// Return the value from an `Indexed` item. Consumes its argument: written with `.map(value)` in mind.
#[allow(clippy::needless_pass_by_value)]
#[inline(always)]
#[must_use]
pub const fn value<Value>(indexed: Indexed<'_, Value>) -> &Value {
    indexed.value
}

/// Clone and return the value from an `Indexed` item. Consumes its argument: written with `.map(value)` in mind.
#[allow(clippy::needless_pass_by_value)]
#[inline(always)]
#[must_use]
pub fn clone_value<Value: Clone>(indexed: Indexed<'_, Value>) -> Value {
    indexed.value.clone()
}

/// Copy and return the value from an `Indexed` item. Consumes its argument: written with `.map(value)` in mind.
#[allow(clippy::needless_pass_by_value)]
#[inline(always)]
#[must_use]
pub const fn copy_value<Value: Copy>(indexed: Indexed<'_, Value>) -> Value {
    *indexed.value
}

/// Split an `Option<Indexed<'a, Value>>` into its index (`Option<usize>`) or value (`Option<&Value>`).
pub trait OptionIndexed<'value> {
    /// The `Value` in `Option<Indexed<'a, Value>>`.
    type Value;

    /// Pull the index out of an `Option<Indexed<'a, Value>>` if it exists.
    #[must_use]
    fn index(&self) -> Option<usize>;

    /// Pull the value out of an `Option<Indexed<'a, Value>>` if it exists.
    #[must_use]
    fn value(&self) -> Option<&'value Self::Value>;
}

impl<'value, Value> OptionIndexed<'value> for Option<Indexed<'value, Value>> {
    type Value = Value;

    #[inline(always)]
    #[must_use]
    fn index(&self) -> Option<usize> {
        self.as_ref().map(|i| i.index)
    }

    #[inline(always)]
    #[must_use]
    fn value(&self) -> Option<&'value Self::Value> {
        self.as_ref().map(|i| i.value)
    }
}
