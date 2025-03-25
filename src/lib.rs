mod finder;

use cfg_if::cfg_if;
use finder::*;
use rayon::prelude::*;

cfg_if! {
    if #[cfg(any(target_feature = "avx2", target_feature = "sse2"))] {
        mod x86_simd;
        use x86_simd::StringPrefix;
        use x86_simd::StringSuffix;
    } else if #[cfg(target_feature = "neon")] {
        mod neon;
        use neon::StringPrefix;
        use neon::StringSuffix;
    } else {
        use finder::StringPrefix;
        use finder::StringSuffix;
    }
}

trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &T) -> Option<&'a T>;
}

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
                let result = match common_prefix {
                    Some(prefix) => F::common(prefix, value.as_ref()),
                    None => Some(value.as_ref()),
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
    use super::*;
    use ya_rand::*;
    use ya_rand_encoding::Base64URL;

    const LEN: usize = 1 << 12;
    const SEARCH_LEN: usize = 69;
    const STRING_LEN: usize = 420;

    #[test]
    fn prefix() {
        let mut rng = new_rng_secure();
        let base = rng.text::<Base64URL>(SEARCH_LEN).unwrap();
        let mut strings = vec![String::with_capacity(SEARCH_LEN + STRING_LEN); LEN];
        strings.iter_mut().for_each(|s| {
            let ext = rng.text::<Base64URL>(STRING_LEN).unwrap();
            s.push_str(&base);
            s.push_str(&ext);
        });
        let prefix = strings.common_prefix().unwrap();
        assert!(base == prefix, "incorrect prefix");
    }

    #[test]
    fn suffix() {
        let mut rng = new_rng_secure();
        let base = rng.text::<Base64URL>(SEARCH_LEN).unwrap();
        let mut strings = vec![String::with_capacity(SEARCH_LEN + STRING_LEN); LEN];
        strings.iter_mut().for_each(|s| {
            let ext = rng.text::<Base64URL>(STRING_LEN).unwrap();
            s.push_str(&ext);
            s.push_str(&base);
        });
        let suffix = strings.common_suffix().unwrap();
        assert!(base == suffix, "incorrect suffix");
    }

    fn random_char(rng: &mut SecureRng) -> char {
        let mut result: char;
        // 2^21 is the lowest power of two above max char value.
        loop {
            match char::from_u32(rng.bits(21) as u32) {
                Some(c) => result = c,
                None => (),
            }
        }
        result
    }
}
