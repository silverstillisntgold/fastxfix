pub trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &'a T) -> &'a T;
}

trait CharEqCounter {
    fn count_eq_chars(self) -> usize;
}

impl<T> CharEqCounter for T
where
    T: Iterator<Item = (char, char)>,
{
    #[inline]
    fn count_eq_chars(self) -> usize {
        self.take_while(|(a, b)| a.eq(b))
            .map(|(a, _)| a.len_utf8())
            .sum()
    }
}

pub struct StringPrefix;
impl Finder<str> for StringPrefix {
    #[inline]
    fn common<'a>(a: &'a str, b: &'a str) -> &'a str {
        let end = a.chars().zip(b.chars()).count_eq_chars();
        unsafe { a.get_unchecked(..end) }
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    #[inline]
    fn common<'a>(a: &'a str, b: &'a str) -> &'a str {
        let end = a.chars().rev().zip(b.chars().rev()).count_eq_chars();
        let begin = a.len() - end;
        unsafe { a.get_unchecked(begin..) }
    }
}

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

pub struct GenericPrefix;
impl<T: Eq> Finder<[T]> for GenericPrefix {
    #[inline]
    fn common<'a>(a: &'a [T], b: &'a [T]) -> &'a [T] {
        let end = a.into_iter().zip(b.into_iter()).count_eq();
        unsafe { a.get_unchecked(..end) }
    }
}

pub struct GenericSuffix;
impl<T: Eq> Finder<[T]> for GenericSuffix {
    #[inline]
    fn common<'a>(a: &'a [T], b: &'a [T]) -> &'a [T] {
        let end = a.into_iter().rev().zip(b.into_iter().rev()).count_eq();
        let begin = a.len() - end;
        unsafe { a.get_unchecked(begin..) }
    }
}
