pub trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &'a T) -> &'a T;
}

pub struct StringPrefix;
impl Finder<str> for StringPrefix {
    #[inline]
    fn common<'a>(a: &'a str, b: &'a str) -> &'a str {
        let end = a.bytes().zip(b.bytes()).count_while_eq();
        unsafe { a.get_unchecked(..end) }
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    #[inline]
    fn common<'a>(a: &'a str, b: &'a str) -> &'a str {
        let end = a.bytes().rev().zip(b.bytes().rev()).count_while_eq();
        let begin = a.len() - end;
        unsafe { a.get_unchecked(begin..) }
    }
}

pub struct GenericPrefix;
impl<T: Eq> Finder<[T]> for GenericPrefix {
    #[inline]
    fn common<'a>(a: &'a [T], b: &'a [T]) -> &'a [T] {
        let end = a.into_iter().zip(b.into_iter()).count_while_eq();
        unsafe { a.get_unchecked(..end) }
    }
}

pub struct GenericSuffix;
impl<T: Eq> Finder<[T]> for GenericSuffix {
    #[inline]
    fn common<'a>(a: &'a [T], b: &'a [T]) -> &'a [T] {
        let end = a
            .into_iter()
            .rev()
            .zip(b.into_iter().rev())
            .count_while_eq();
        let begin = a.len() - end;
        unsafe { a.get_unchecked(begin..) }
    }
}

trait EqCounter {
    fn count_while_eq(self) -> usize;
}

impl<T, U> EqCounter for T
where
    T: Iterator<Item = (U, U)>,
    U: Eq,
{
    #[inline]
    fn count_while_eq(self) -> usize {
        self.take_while(|(a, b)| a.eq(b)).count()
    }
}
