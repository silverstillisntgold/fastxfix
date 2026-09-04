/*!
# FastXFix

**This crate is considered feature complete.**

A small utility crate for finding the longest common prefix/suffix of 2D collections at
absolutely insane speeds, made possible by [`rayon`] and SIMD optimizations.

"2D collections" refers to arrangements like `Vec<T>`, `HashSet<T>`, or `LinkedList<T>`.
When `T` implements `AsRef<str>`, you'll be able to use the methods of [`CommonStr`] on it.
When `T` implements `AsRef<&[U]>` (meaning that `T` is a slice of some kind) then you'll have
access to the methods of [`CommonRaw`]. These two conditions are not mutually exclusive, so
it's up to the user to ensure they're using the method that best coincides with what they're
trying to accomplish.

If you're trying to extract information about strings, **always** prefer using [`CommonStr`]
methods: they are specifically optimized for handling rust's UTF-8 encoded strings.

## Examples

```
use fastxfix::CommonStr;
use std::num::NonZeroUsize;

let s1 = "wowie_this_is_a_string".to_string();
let s2 = "wowie_this_is_another_string_".to_string();

let v = vec![s1, s2];
let common_prefix = v.common_prefix().expect("we know there is a common prefix");
let len: NonZeroUsize = v.common_prefix_len().expect("we know there is a common prefix");
assert!(common_prefix.len() == len.get());
// The strings have no common suffix.
assert!(v.common_suffix_len().is_none());
```
*/

#![forbid(missing_docs)]

use finder::*;
use rayon::prelude::*;
use std::num::NonZeroUsize;

mod finder;

/// Trait for finding the longest common [`String`] prefix/suffix of any 2D collection.
pub trait CommonStr {
    /// Returns the longest common prefix of all referenced strings.
    ///
    /// Returns `None` when there is no common prefix.
    fn common_prefix(&self) -> Option<String> {
        self.common_prefix_ref().map(|s| s.to_string())
    }

    /// Returns the longest common suffix of all referenced strings.
    ///
    /// Returns `None` when there is no common suffix.
    fn common_suffix(&self) -> Option<String> {
        self.common_suffix_ref().map(|s| s.to_string())
    }

    /// Returns the length of the longest common prefix of all referenced strings.
    ///
    /// Returns `None` instead of 0 when there is no common prefix.
    fn common_prefix_len(&self) -> Option<NonZeroUsize> {
        self.common_prefix_ref()
            .and_then(|s| NonZeroUsize::new(s.len()))
    }

    /// Returns the length of the longest common suffix of all referenced strings.
    ///
    /// Returns `None` instead of 0 when there is no common suffix.
    fn common_suffix_len(&self) -> Option<NonZeroUsize> {
        self.common_suffix_ref()
            .and_then(|s| NonZeroUsize::new(s.len()))
    }

    /// Returns a reference to the string which has the longest common
    /// prefix of all strings in the collection.
    ///
    /// Returns `None` when there is no common prefix.
    fn common_prefix_ref(&self) -> Option<&str>;

    /// Returns a reference to the string which has the longest common
    /// suffix of all strings in the collection.
    ///
    /// Returns `None` when there is no common suffix.
    fn common_suffix_ref(&self) -> Option<&str>;
}

/// Trait for finding the longest common raw prefix/suffix of any 2D collection.
pub trait CommonRaw<T: Clone> {
    /// Returns the longest common prefix of all referenced data.
    ///
    /// Returns `None` when there is no common prefix.
    fn common_prefix_raw(&self) -> Option<Vec<T>> {
        self.common_prefix_raw_ref().map(|s| s.to_vec())
    }

    /// Returns the longest common suffix of all referenced data.
    ///
    /// Returns `None` when there is no common suffix.
    fn common_suffix_raw(&self) -> Option<Vec<T>> {
        self.common_suffix_raw_ref().map(|s| s.to_vec())
    }

    /// Returns the length of the longest common prefix of all referenced data.
    ///
    /// Returns `None` instead of 0 when there is no common prefix.
    fn common_prefix_raw_len(&self) -> Option<NonZeroUsize> {
        self.common_prefix_raw_ref()
            .and_then(|s| NonZeroUsize::new(s.len()))
    }

    /// Returns the length of the longest common suffix of all referenced data.
    ///
    /// Returns `None` instead of 0 when there is no common suffix.
    fn common_suffix_raw_len(&self) -> Option<NonZeroUsize> {
        self.common_suffix_raw_ref()
            .and_then(|s| NonZeroUsize::new(s.len()))
    }

    /// Returns a reference to the element which has the longest common
    /// prefix of all data in the collection.
    ///
    /// Returns `None` when there is no common prefix.
    fn common_prefix_raw_ref(&self) -> Option<&[T]>;

    /// Returns a reference to the element which has the longest common
    /// suffix of all data in the collection.
    ///
    /// Returns `None` when there is no common suffix.
    fn common_suffix_raw_ref(&self) -> Option<&[T]>;
}

impl<C: ?Sized, T> CommonStr for C
where
    for<'a> &'a C: IntoParallelIterator<Item = &'a T>,
    T: AsRef<str>,
{
    fn common_prefix_ref(&self) -> Option<&str> {
        find_common::<_, StringPrefix, _, _>(self)
    }

    fn common_suffix_ref(&self) -> Option<&str> {
        find_common::<_, StringSuffix, _, _>(self)
    }
}

impl<C: ?Sized, T, U> CommonRaw<U> for C
where
    for<'a> &'a C: IntoParallelIterator<Item = &'a T>,
    T: AsRef<[U]>,
    U: Clone + Eq + Sync,
{
    fn common_prefix_raw_ref(&self) -> Option<&[U]> {
        find_common::<_, GenericPrefix, _, _>(self)
    }

    fn common_suffix_raw_ref(&self) -> Option<&[U]> {
        find_common::<_, GenericSuffix, _, _>(self)
    }
}

/// Core function for finding LCP or LCS.
///
/// The idea is to, for each pair of referenced values, compute the
/// result of [`Finder::common`] and pass it along to be one of
/// the values in the next pair. At any point, that result might be `None`,
/// (there was no common prefix/suffix), causing the routine to terminate
/// as soon as rayon is able to halt execution across it's threadpool.
fn find_common<C: ?Sized, F, T, U>(collection: &C) -> Option<&U>
where
    for<'a> &'a C: IntoParallelIterator<Item = &'a T>,
    F: Finder<U>,
    T: AsRef<U>,
    U: ?Sized + Sync,
{
    collection
        .into_par_iter()
        .map(|current| Some(current.as_ref()))
        .try_reduce_with(F::common)
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::{CommonRaw, CommonStr};
    use std::hint::black_box;
    use std::iter;
    use ya_rand::*;

    const BASE_LEN: usize = 19;
    const COMMON: &str = "this is just a simple sentence";
    const EXT_LEN: usize = 13;
    const TOTAL_LEN: usize = BASE_LEN + EXT_LEN;
    const VEC_LEN: usize = 1 << 15;

    #[test]
    fn str_prefix_sanity() {
        let v = black_box(vec![COMMON.to_string(); BASE_LEN]);
        let common = v.common_prefix_ref().unwrap();
        assert_eq!(common, COMMON);
        assert_eq!(common.len(), COMMON.len());
    }

    #[test]
    fn str_suffix_sanity() {
        let v = black_box(vec![COMMON.to_string(); BASE_LEN]);
        let common = v.common_suffix_ref().unwrap();
        assert_eq!(common, COMMON);
        assert_eq!(common.len(), COMMON.len());
    }

    #[test]
    fn raw_prefix_sanity() {
        let v = black_box(vec![COMMON.as_bytes().to_vec(); BASE_LEN]);
        let common = v.common_prefix_raw_ref().unwrap();
        assert_eq!(common, COMMON.as_bytes());
        assert_eq!(common.len(), COMMON.len());
    }

    #[test]
    fn raw_suffix_sanity() {
        let v = black_box(vec![COMMON.as_bytes().to_vec(); BASE_LEN]);
        let common = v.common_suffix_raw_ref().unwrap();
        assert_eq!(common, COMMON.as_bytes());
        assert_eq!(common.len(), COMMON.len());
    }

    #[test]
    fn misc() {
        let input: [String; 0] = [];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["just a single entry"];
        let prefix = input.common_prefix().unwrap();
        assert_eq!(prefix, input[0]);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, input[0]);

        let input = ["foobar", "fooqux", "foodle", "fookys"];
        let prefix = input.common_prefix().unwrap();
        assert_eq!(prefix, "foo");
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["café", "caféine"];
        let prefix = input.common_prefix().unwrap();
        assert_eq!(prefix, "café");
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["äbc", "âbc"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "bc");

        let input = ["abc€", "xyz€"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "€");

        let input = ["abcä", "defâ"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["some thingy", "nothing"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["-lol-", "_lol_"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["a🤖b", "a🤡b"];
        let prefix = input.common_prefix().unwrap();
        assert_eq!(prefix, "a");
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "b");

        let input = ["résumé", "résister"];
        let prefix = input.common_prefix().unwrap();
        assert_eq!(prefix, "rés");
        let suffix = input.common_suffix();
        assert_eq!(suffix, None);

        let input = ["abcédef", "xyzèdef"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "def");

        let input = ["Goodbye 👋", "Farewell 👋"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, " 👋");

        let input = ["Family: 👨‍👩‍👧", "Group: 👨‍👩‍👧"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, ": 👨‍👩‍👧");

        let input = ["just some words 世界", "世界"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "世界");

        let input = ["tests😀", "best😀"];
        let prefix = input.common_prefix();
        assert_eq!(prefix, None);
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "😀");

        let input = ["wowie_bruhther_clap", "wowie-lol-clap", "wowie_xd_clap"];
        let prefix = input.common_prefix().unwrap();
        assert_eq!(prefix, "wowie");
        let suffix = input.common_suffix().unwrap();
        assert_eq!(suffix, "clap");
    }

    #[test]
    fn prefix_ascii() {
        let mut rng = new_rng();
        let base = new_string_with::<BASE_LEN, _>(|| random_ascii(&mut rng));
        let mut strings = vec![String::with_capacity(TOTAL_LEN); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string_with::<EXT_LEN, _>(|| random_ascii(&mut rng));
            s.push_str(&base);
            s.push_str(&ext);
        });
        let prefix = strings.common_prefix().unwrap();
        assert_eq!(base, prefix);
    }

    #[test]
    fn suffix_ascii() {
        let mut rng = new_rng();
        let base = new_string_with::<BASE_LEN, _>(|| random_ascii(&mut rng));
        let mut strings = vec![String::with_capacity(TOTAL_LEN); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string_with::<EXT_LEN, _>(|| random_ascii(&mut rng));
            s.push_str(&ext);
            s.push_str(&base);
        });
        let suffix = strings.common_suffix().unwrap();
        assert_eq!(base, suffix);
    }

    fn random_ascii(rng: &mut ShiroRng) -> char {
        rng.bits(7) as u8 as char
    }

    #[test]
    fn prefix_char() {
        let mut rng = new_rng();
        let base = new_string_with::<BASE_LEN, _>(|| random_char(&mut rng));
        let mut strings = vec![String::with_capacity(TOTAL_LEN * 4); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string_with::<EXT_LEN, _>(|| random_char(&mut rng));
            s.push_str(&base);
            s.push_str(&ext);
        });
        let prefix = strings.common_prefix().unwrap();
        assert_eq!(base, prefix);
    }

    #[test]
    fn suffix_char() {
        let mut rng = new_rng();
        let base = new_string_with::<BASE_LEN, _>(|| random_char(&mut rng));
        let mut strings = vec![String::with_capacity(TOTAL_LEN * 4); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string_with::<EXT_LEN, _>(|| random_char(&mut rng));
            s.push_str(&ext);
            s.push_str(&base);
        });
        let suffix = strings.common_suffix().unwrap();
        assert_eq!(base, suffix);
    }

    fn random_char(rng: &mut ShiroRng) -> char {
        loop {
            // 2^21 is the smallest power-of-two value outside of
            // the maximum valid UTF-8 character range.
            let val = rng.bits(21) as u32;
            match char::from_u32(val) {
                Some(c) => return c,
                None => continue,
            }
        }
    }

    fn new_string_with<const SIZE: usize, F>(f: F) -> String
    where
        F: FnMut() -> char,
    {
        iter::repeat_with(f).take(SIZE).collect()
    }

    #[test]
    fn prefix_generic() {
        let mut rng = new_rng();
        let base = new_vec_with::<BASE_LEN, _>(|| rng.u64());
        let mut nested = vec![Vec::new(); VEC_LEN];
        nested.iter_mut().for_each(|cur| {
            let ext = new_vec_with::<EXT_LEN, _>(|| rng.u64());
            cur.extend_from_slice(&base);
            cur.extend_from_slice(&ext);
        });
        let prefix = nested.common_prefix_raw().unwrap();
        assert_eq!(base, prefix);
    }

    #[test]
    fn suffix_generic() {
        let mut rng = new_rng();
        let base = new_vec_with::<BASE_LEN, _>(|| rng.u64());
        let mut nested = vec![Vec::new(); VEC_LEN];
        nested.iter_mut().for_each(|cur| {
            let ext = new_vec_with::<EXT_LEN, _>(|| rng.u64());
            cur.extend_from_slice(&ext);
            cur.extend_from_slice(&base);
        });
        let prefix = nested.common_suffix_raw().unwrap();
        assert_eq!(base, prefix);
    }

    fn new_vec_with<const SIZE: usize, F>(f: F) -> Vec<u64>
    where
        F: FnMut() -> u64,
    {
        iter::repeat_with(f).take(SIZE).collect()
    }

    #[test]
    fn singleton_empty_len_is_none() {
        let strings = [""];
        assert_eq!(strings.common_prefix_len(), None);
        assert_eq!(strings.common_suffix_len(), None);

        let raw = vec![Vec::<u8>::new()];
        assert_eq!(raw.common_prefix_raw_len(), None);
        assert_eq!(raw.common_suffix_raw_len(), None);
    }
}
