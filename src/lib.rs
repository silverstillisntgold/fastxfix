mod finder;

use finder::*;
use rayon::prelude::*;

pub trait CommonStr {
    /// Returns the longest common prefix of all referenced strings.
    fn common_prefix(&self) -> Option<String>;
    /// Returns the longest common suffix of all referenced strings.
    fn common_suffix(&self) -> Option<String>;
    /// Returns the length of the longest common prefix of all referenced strings.
    fn common_prefix_len(&self) -> usize;
    /// Returns the length of the longest common suffix of all referenced strings.
    fn common_suffix_len(&self) -> usize;
}

pub trait CommonRaw<T> {
    /// Returns the longest common prefix of all referenced data.
    fn common_prefix_raw(&self) -> Option<Vec<T>>;
    /// Returns the longest common suffix of all referenced data.
    fn common_suffix_raw(&self) -> Option<Vec<T>>;
    /// Returns the length of the longest common prefix of all referenced data.
    fn common_prefix_raw_len(&self) -> usize;
    /// Returns the length of the longest common suffix of all referenced data.
    fn common_suffix_raw_len(&self) -> usize;
}

impl<C: ?Sized, T> CommonStr for C
where
    for<'a> &'a C: IntoParallelIterator<Item = &'a T>,
    T: AsRef<str> + Sync,
{
    #[inline(never)]
    fn common_prefix(&self) -> Option<String> {
        find_common::<_, StringPrefix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_suffix(&self) -> Option<String> {
        find_common::<_, StringSuffix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_prefix_len(&self) -> usize {
        find_common::<_, StringPrefix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }

    #[inline(never)]
    fn common_suffix_len(&self) -> usize {
        find_common::<_, StringSuffix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }
}

impl<C: ?Sized, T, U> CommonRaw<U> for C
where
    for<'a> &'a C: IntoParallelIterator<Item = &'a T>,
    T: AsRef<[U]> + Sync,
    U: Clone + Eq + Sync,
{
    #[inline(never)]
    fn common_prefix_raw(&self) -> Option<Vec<U>> {
        find_common::<_, GenericPrefix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_suffix_raw(&self) -> Option<Vec<U>> {
        find_common::<_, GenericSuffix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_prefix_raw_len(&self) -> usize {
        find_common::<_, GenericPrefix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }

    #[inline(never)]
    fn common_suffix_raw_len(&self) -> usize {
        find_common::<_, GenericSuffix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }
}

/// Core function for finding LCP or LCS. It looks a bit involved,
/// but most of what goes on in here is just to ensure we satisfy the
/// type constraints laid out by rayon.
///
/// The core idea is to, for each pair of referenced values, compute the
/// result of `Finder::common` and pass that result along to be one of
/// the values in the next pair. At any point, that result might be `None`,
/// (there was no common prefix/suffix), and the routine will terminate
/// as soon as rayon is able to halt execution.
fn find_common<C: ?Sized, F, T, U>(collection: &C) -> Option<&U>
where
    for<'a> &'a C: IntoParallelIterator<Item = &'a T>,
    F: Finder<U>,
    T: AsRef<U> + Sync,
    U: ?Sized + Sync,
{
    // We have to use the `try_*` variants of fold/reduce so we can fail
    // early when any two items don't have a common prefix/suffix.
    collection
        .into_par_iter()
        .try_fold(
            || None,
            |previous, current| {
                let c_ref = current.as_ref();
                let result = match previous {
                    Some(prefix) => F::common(prefix, c_ref),
                    None => Some(c_ref),
                }?;
                Some(Some(result))
            },
        )
        .try_reduce(
            || None,
            |a, b| {
                let result = match (a, b) {
                    (Some(a), Some(b)) => F::common(a, b),
                    (Some(c), None) | (None, Some(c)) => Some(c),
                    (None, None) => None,
                }?;
                Some(Some(result))
            },
        )
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::CommonStr;
    use ya_rand::*;

    const VEC_LEN: usize = 1 << 16;
    const BASE_LEN: usize = 19;
    const EXT_LEN: usize = 13;
    const TOTAL_LEN: usize = BASE_LEN + EXT_LEN;

    #[test]
    fn miscellaneous() {
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
        let mut rng = new_rng_secure();
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
        let mut rng = new_rng_secure();
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

    #[test]
    fn prefix_char() {
        let mut rng = new_rng_secure();
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
        let mut rng = new_rng_secure();
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

    #[inline(always)]
    fn new_string_with<const SIZE: usize, F>(f: F) -> String
    where
        F: FnMut() -> char,
    {
        core::iter::repeat_with(f).take(SIZE).collect()
    }

    #[inline(always)]
    fn random_ascii(rng: &mut SecureRng) -> char {
        rng.bits(7) as u8 as char
    }

    #[inline(always)]
    fn random_char(rng: &mut SecureRng) -> char {
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
}
