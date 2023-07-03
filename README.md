# Reiterator

You know that one friend who can tell the same story over and over again, but you only really listen the first time? Maybe they even have a _set_ of stories so interesting you only really needed to listen to it once?

This crate is that friend.

## What?

This `no_std` crate takes an iterator and caches its output, allowing you to access an immutable reference to the output of any referentially transparent iterator call.
Rewind it or set it ten elements ahead, and it'll gladly oblige, but only when you ask it. A little taste of lazy evaluation in eager-flavored Rust.
Plus, it returns the index of the value as well, but there are built-in `map`-compatible mini-functions to get either the value or the index only:

```rust
use reiterator::{Reiterate, indexed::Indexed};

let mut iter = vec!['a', 'b', 'c'].reiterate(); // None of the values are computed or cached until...

// You can drive it like a normal `Iterator`:
assert_eq!(iter.next(), Some(Indexed { index: 0, value: &'a' })); // here: only the first one, whose cache is referenced.
assert_eq!(iter.next(), Some(Indexed { index: 1, value: &'b' })); // Cooked up the second value on demand.
assert_eq!(iter.next(), Some(Indexed { index: 2, value: &'c' }));
assert_eq!(iter.next(), None); // Out of bounds!

// Note that we literally return the same memory addresses as before:
iter.restart();
assert_eq!(iter.next(), Some(Indexed { index: 0, value: &'a' }));
assert_eq!(iter.next(), Some(Indexed { index: 1, value: &'b' }));

// Start from anywhere:
iter.index.set(1);
assert_eq!(iter.next(), Some(Indexed { index: 1, value: &'b' }));
assert_eq!(iter.next(), Some(Indexed { index: 2, value: &'c' }));

// Or hop around at will, just as quickly (if not faster):
assert_eq!(iter.at(1), Some(&'b'));
assert_eq!(iter.at(2), Some(&'c'));
assert_eq!(iter.at(3), None);
```

## An entire crate just to do that?

Yeah, I know. It's pretty simple. But the lifetimes and edge cases are hard to get right and easy to overlook, respectively.

Think of it as transferring all the pain you would have experienced down the road and transferring it to me, so fewer people overall in the world have to suffer.
Utilitarian, really.
