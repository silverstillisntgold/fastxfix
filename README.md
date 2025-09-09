# FastXFix

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

[`rayon`]: https://crates.io/crates/rayon
[`CommonStr`]: https://docs.rs/fastxfix/latest/fastxfix/trait.CommonStr.html
[`CommonRaw`]: https://docs.rs/fastxfix/latest/fastxfix/trait.CommonRaw.html

## Examples

```rust
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
