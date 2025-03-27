mod finder;

use finder::*;
use rayon::prelude::*;

pub trait CommonStr {
    /// Returns the longest common prefix of all referenced strings.
    fn common_prefix(&self) -> Option<String>;

    /// Returns the longest common suffix of all referenced strings.
    fn common_suffix(&self) -> Option<String>;

    /// Returns the length of the longest common prefix of all strings.
    fn common_prefix_len(&self) -> usize;

    /// Returns the length of the longest common suffix of all strings.
    fn common_suffix_len(&self) -> usize;
}

impl<T> CommonStr for [T]
where
    T: AsRef<str> + Sync,
{
    #[inline(never)]
    fn common_prefix(&self) -> Option<String> {
        find_common::<StringPrefix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_suffix(&self) -> Option<String> {
        find_common::<StringSuffix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_prefix_len(&self) -> usize {
        find_common::<StringPrefix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }

    #[inline(never)]
    fn common_suffix_len(&self) -> usize {
        find_common::<StringSuffix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }
}

pub trait CommonRaw<T> {
    fn common_prefix_raw(&self) -> Option<Vec<T>>;

    fn common_suffix_raw(&self) -> Option<Vec<T>>;

    fn common_prefix_raw_len(&self) -> usize;

    fn common_suffix_raw_len(&self) -> usize;
}

impl<T, U> CommonRaw<U> for [T]
where
    T: AsRef<[U]> + Sync,
    U: Clone + Eq + Sync,
{
    #[inline(never)]
    fn common_prefix_raw(&self) -> Option<Vec<U>> {
        find_common::<GenericPrefix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_suffix_raw(&self) -> Option<Vec<U>> {
        find_common::<GenericSuffix, _, _>(self).map(|s| s.to_owned())
    }

    #[inline(never)]
    fn common_prefix_raw_len(&self) -> usize {
        find_common::<GenericPrefix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }

    #[inline(never)]
    fn common_suffix_raw_len(&self) -> usize {
        find_common::<GenericSuffix, _, _>(self)
            .map(|s| s.len())
            .unwrap_or_default()
    }
}

fn find_common<F, T, U>(slice: &[T]) -> Option<&U>
where
    F: Finder<U>,
    T: AsRef<U> + Sync,
    U: ?Sized + Sync,
{
    slice
        .into_par_iter()
        .try_fold(
            || None,
            |common_prefix, value| {
                let v_ref = value.as_ref();
                let result = match common_prefix {
                    Some(prefix) => F::common(prefix, v_ref),
                    None => Some(v_ref),
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
    fn misc_tests() {
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
    }

    #[test]
    fn prefix_ascii_rand() {
        let mut rng = new_rng_secure();
        let base = new_string::<BASE_LEN, _>(|| rng.bits(7) as u8 as char);
        let mut strings = vec![String::with_capacity(TOTAL_LEN); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string::<EXT_LEN, _>(|| rng.bits(7) as u8 as char);
            s.push_str(&base);
            s.push_str(&ext);
        });
        let prefix = strings.common_prefix().unwrap();
        assert_eq!(base, prefix);
    }

    #[test]
    fn suffix_ascii_rand() {
        let mut rng = new_rng_secure();
        let base = new_string::<BASE_LEN, _>(|| rng.bits(7) as u8 as char);
        let mut strings = vec![String::with_capacity(TOTAL_LEN); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string::<EXT_LEN, _>(|| rng.bits(7) as u8 as char);
            s.push_str(&ext);
            s.push_str(&base);
        });
        let suffix = strings.common_suffix().unwrap();
        assert_eq!(base, suffix);
    }

    #[test]
    fn prefix_char_rand() {
        let mut rng = new_rng_secure();
        let base = new_string::<BASE_LEN, _>(|| random_char(&mut rng));
        let mut strings = vec![String::with_capacity(TOTAL_LEN * 4); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string::<EXT_LEN, _>(|| random_char(&mut rng));
            s.push_str(&base);
            s.push_str(&ext);
        });
        let prefix = strings.common_prefix().unwrap();
        assert_eq!(base, prefix);
    }

    #[test]
    fn suffix_char_rand() {
        let mut rng = new_rng_secure();
        let base = new_string::<BASE_LEN, _>(|| random_char(&mut rng));
        let mut strings = vec![String::with_capacity(TOTAL_LEN * 4); VEC_LEN];
        strings.iter_mut().for_each(|s| {
            let ext = new_string::<EXT_LEN, _>(|| random_char(&mut rng));
            s.push_str(&ext);
            s.push_str(&base);
        });
        let prefix = strings.common_suffix().unwrap();
        assert_eq!(base, prefix);
    }

    #[inline(always)]
    fn random_char(rng: &mut SecureRng) -> char {
        loop {
            let val = rng.bits(21) as u32;
            match char::from_u32(val) {
                Some(c) => return c,
                None => continue,
            }
        }
    }

    #[inline(always)]
    fn new_string<const SIZE: usize, F>(f: F) -> String
    where
        F: FnMut() -> char,
    {
        core::iter::repeat_with(f).take(SIZE).collect()
    }
}
