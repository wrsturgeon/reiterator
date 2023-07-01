# Reiterator

You know that one friend who can tell the same story over and over again, but you only really listen the first time? Maybe they even have a _set_ of stories so interesting you only really needed to listen to it once?

This crate is that friend.

## What?

This `no_std` crate takes an iterator and caches its output, allowing you to access an immutable reference to the output of any referentially transparent iterator call.
Rewind it or set it ten elements ahead, and it'll gladly oblige, but only when you ask it. A little taste of lazy evaluation in eager-flavored Rust.
Plus, it returns the index of the value as well, but there are built-in `map`-compatible mini-functions to get either the value or the index only:

```rust
use reiterator::{Reiterate, value};
let mut iter = vec!['a', 'b', 'c'].reiterate(); // None of the values are computed or cached until...
//              vvvvv here. And, even then, only the first two.
assert_eq!(iter.at(1).map(value), Some(&'b'));
//                             ^^^^^ And an analogous `index` function.
```

You can drive it like a normal `Iterator`:

```rust
use reiterator::{Indexed, Reiterate, value};
let mut iter = vec!['a', 'b', 'c'].reiterate(); // None of the values are computed or cached until...
let first_a = iter.get();
let mut iter = vec!['a', 'b', 'c'].reiterate(); // None of the values are computed or cached until...
assert_eq!(iter.get(), Some(Indexed { index: 0, value: &'a' }));
assert_eq!(iter.get(), Some(Indexed { index: 0, value: &'a' })); // Using the cached version
assert_eq!(iter.next(), Some(Indexed { index: 0, value: &'a' })); // Note that `next` doesn't "know" we've already called `get`
assert_eq!(iter.get(), Some(Indexed { index: 1, value: &'b' })); // ...but it does change the internal `next_index`
assert_eq!(iter.next(), Some(Indexed { index: 1, value: &'b' }));
assert_eq!(iter.next(), Some(Indexed { index: 2, value: &'c' }));
assert_eq!(iter.next(), None);

// But then you can rewind and do it all again for free, returning cached references to the same values we just made:
iter.restart();
assert_eq!(iter.next(), Some(Indexed { index: 0, value: &'a' }));
assert_eq!(iter.next(), Some(Indexed { index: 1, value: &'b' }));
assert_eq!(iter.next(), Some(Indexed { index: 2, value: &'c' }));
assert_eq!(iter.next(), None);

// Or start from anywhere:
iter.next_index = 1;
assert_eq!(iter.next(), Some(Indexed { index: 1, value: &'b' }));
assert_eq!(iter.next(), Some(Indexed { index: 2, value: &'c' }));
assert_eq!(iter.next(), None);

// And just to prove that it's literally a reference to the same cached value in memory:
assert!(std::ptr::eq(first_a.unwrap().value, iter.at(0).unwrap().value));
```

## An entire crate just to do that?

Yeah, I know. It's pretty simple. But the lifetimes and edge cases are hard to get right and easy to overlook, respectively.

Think of it as transferring all the pain you would have experienced down the road and transferring it to me, so fewer people overall in the world have to suffer.
Utilitarian, really.
