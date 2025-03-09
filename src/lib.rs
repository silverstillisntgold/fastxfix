mod finder;

use finder::*;
use rayon::prelude::*;

pub trait CommonStr {
    /// Returns the longest common prefix of all strings.
    fn common_prefix(&self) -> Option<String>;

    /// Returns the longest common suffix of all strings.
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

#[inline]
fn find_common<F, T, U>(slice: &[T]) -> Option<&U>
where
    F: Finder<U>,
    T: AsRef<U> + Sync,
    U: ?Sized + Sync,
{
    slice
        .into_par_iter()
        .map(|v| v.as_ref())
        .reduce_with(|common, cur| F::common(common, cur))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ya_rand::*;
    use ya_rand_encoding::Base64URL as Encoder;

    const SEARCH_LEN: usize = 69;
    const STRING_LEN: usize = 420;
    const LEN: usize = 1 << 12;

    #[test]
    fn prefix() {
        let mut rng = new_rng_secure();
        let base_prefix = rng.text::<Encoder>(SEARCH_LEN).unwrap();
        let mut strings = vec![String::with_capacity(SEARCH_LEN + STRING_LEN); LEN];
        strings.iter_mut().for_each(|s| {
            s.push_str(&base_prefix);
            let suffix = rng.text::<Encoder>(STRING_LEN).unwrap();
            s.push_str(&suffix);
        });
        let prefix = strings.common_prefix().expect("should be `Some`");
        assert!(prefix == base_prefix, "incorrect prefix");
    }

    #[test]
    fn suffix() {
        let mut rng = new_rng_secure();
        let base_suffix = rng.text::<Encoder>(SEARCH_LEN).unwrap();
        let mut strings = vec![String::with_capacity(SEARCH_LEN + STRING_LEN); LEN];
        strings.iter_mut().for_each(|s| {
            let prefix = rng.text::<Encoder>(STRING_LEN).unwrap();
            s.push_str(&prefix);
            s.push_str(&base_suffix);
        });
        let prefix = strings.common_suffix().expect("should be `Some`");
        assert!(prefix == base_suffix, "incorrect suffix");
    }
}
