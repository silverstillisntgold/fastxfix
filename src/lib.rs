mod finder;

use finder::*;
use rayon::prelude::*;

pub trait CommonStr {
    /// Returns the longest common prefix of all strings.
    fn common_prefix(&self) -> Option<String>;

    /// Returns the longest common suffix of all strings.
    fn common_suffix(&self) -> Option<String>;

    /// Returns the length of the longest common prefix of all strings.
    fn common_prefix_len(&self) -> Option<usize>;

    /// Returns the length of the longest common suffix of all strings.
    fn common_suffix_len(&self) -> Option<usize>;
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
    fn common_prefix_len(&self) -> Option<usize> {
        find_common::<StringPrefix, _, _>(self).map(|s| s.len())
    }

    #[inline(never)]
    fn common_suffix_len(&self) -> Option<usize> {
        find_common::<StringSuffix, _, _>(self).map(|s| s.len())
    }
}

pub trait CommonRaw<T> {
    fn common_prefix_raw(&self) -> Option<Vec<T>>;

    fn common_suffix_raw(&self) -> Option<Vec<T>>;

    fn common_prefix_raw_len(&self) -> Option<usize>;

    fn common_suffix_raw_len(&self) -> Option<usize>;
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
    fn common_prefix_raw_len(&self) -> Option<usize> {
        find_common::<GenericPrefix, _, _>(self).map(|s| s.len())
    }

    #[inline(never)]
    fn common_suffix_raw_len(&self) -> Option<usize> {
        find_common::<GenericSuffix, _, _>(self).map(|s| s.len())
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
        // TODO: Possible to directly use `reduce` instead?
        .reduce_with(|common, cur| F::common(common, cur))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hint::black_box;

    const COMMON: &str = "愛 This is the common SHITE xD 愛";
    const SIZE: usize = 1 << 12;

    #[test]
    fn prefix() {
        let mut vec = vec![String::with_capacity(256); SIZE];
        let mut i = black_box(SIZE);
        vec.iter_mut().for_each(|v| {
            let s = i.to_string();
            v.push_str(COMMON);
            v.push_str(s.as_str());
            i += 1;
        });
        let prefix = vec.common_prefix();
        assert_ne!(prefix, None, "prefix should be Some(_)");
        let prefix = prefix.unwrap();
        assert_eq!(prefix, COMMON, "incorrect prefix");
    }

    #[test]
    fn suffix() {
        let mut vec = vec![String::with_capacity(256); SIZE];
        let mut i = black_box(SIZE);
        vec.iter_mut().for_each(|v| {
            let s = i.to_string();
            v.push_str(s.as_str());
            v.push_str(COMMON);
            i += 1;
        });
        let suffix = vec.common_suffix();
        assert_ne!(suffix, None, "suffix should be Some(_)");
        let suffix = suffix.unwrap();
        assert_eq!(suffix, COMMON, "incorrect suffix");
    }
}
