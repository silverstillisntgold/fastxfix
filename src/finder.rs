use crate::Finder;

trait EqCounter {
    fn count_eq(self) -> usize;
}

impl<T, U> EqCounter for T
where
    T: Iterator<Item = (U, U)>,
    U: Eq,
{
    #[inline]
    fn count_eq(self) -> usize {
        self.take_while(|(a, b)| a.eq(b)).count()
    }
}

pub struct StringPrefix;
impl Finder<str> for StringPrefix {
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let a_iter = a.bytes();
        let b_iter = b.bytes();
        let mut end = a_iter.zip(b_iter).count_eq();
        match end != 0 {
            true => Some({
                while !a.is_char_boundary(end) {
                    end -= 1;
                }
                unsafe { a.get_unchecked(..end) }
            }),
            false => None,
        }
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let a_iter = a.bytes().rev();
        let b_iter = b.bytes().rev();
        let end = a_iter.zip(b_iter).count_eq();
        match end != 0 {
            true => Some({
                let mut begin = a.len() - end;
                while !a.is_char_boundary(begin) {
                    begin += 1;
                }
                unsafe { a.get_unchecked(begin..) }
            }),
            false => None,
        }
    }
}

pub struct GenericPrefix;
impl<T> Finder<[T]> for GenericPrefix
where
    T: Eq,
{
    fn common<'a>(a: &'a [T], b: &[T]) -> Option<&'a [T]> {
        let a_iter = a.into_iter();
        let b_iter = b.into_iter();
        let end = a_iter.zip(b_iter).count_eq();
        match end != 0 {
            true => Some(unsafe { a.get_unchecked(..end) }),
            false => None,
        }
    }
}

pub struct GenericSuffix;
impl<T> Finder<[T]> for GenericSuffix
where
    T: Eq,
{
    fn common<'a>(a: &'a [T], b: &[T]) -> Option<&'a [T]> {
        let a_iter = a.into_iter().rev();
        let b_iter = b.into_iter().rev();
        let end = a_iter.zip(b_iter).count_eq();
        match end != 0 {
            true => Some({
                let begin = a.len() - end;
                unsafe { a.get_unchecked(begin..) }
            }),
            false => None,
        }
    }
}

#[allow(unused)]
#[inline]
pub fn finalize_prefix(s: &str, end: isize) -> Option<&str> {
    let mut end = end as usize;
    match end != 0 {
        true => Some({
            while !s.is_char_boundary(end) {
                end = end.wrapping_sub(1);
            }
            unsafe { s.get_unchecked(..end) }
        }),
        false => None,
    }
}

#[allow(unused)]
#[inline]
pub fn finalize_suffix(s: &str, begin: isize) -> Option<&str> {
    let mut begin = begin as usize;
    match begin != s.len() {
        true => Some({
            while !s.is_char_boundary(begin) {
                begin = begin.wrapping_add(1);
            }
            unsafe { s.get_unchecked(begin..) }
        }),
        false => None,
    }
}
