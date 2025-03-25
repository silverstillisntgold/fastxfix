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
    use super::*;
    use ya_rand::*;

    const VEC_LEN: usize = 1 << 19;
    const BASE_LEN: usize = 6;
    const EXT_LEN: usize = 9;
    const TOTAL_LEN: usize = BASE_LEN + EXT_LEN;

    #[test]
    fn prefix_ascii() {
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
    fn suffix_ascii() {
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
    fn prefix_char() {
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
    fn suffix_char() {
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
                None => (),
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
