pub trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &T) -> Option<&'a T>;
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
    #[inline]
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let a_iter = a.chars();
        let b_iter = b.chars();
        let end = a_iter.zip(b_iter).count_eq_chars();
        match end != 0 {
            true => Some(unsafe { a.get_unchecked(..end) }),
            false => None,
        }
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    #[inline]
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let a_iter = a.chars().rev();
        let b_iter = b.chars().rev();
        let end = a_iter.zip(b_iter).count_eq_chars();
        match end != 0 {
            true => Some({
                let begin = a.len() - end;
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
    #[inline]
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
    #[inline]
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
