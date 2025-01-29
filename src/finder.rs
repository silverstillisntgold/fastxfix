pub trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &'a T) -> &'a T;

    fn is_empty(instance: &T) -> bool;
}

pub struct StringPrefix;
impl Finder<str> for StringPrefix {
    #[inline]
    fn common<'a>(a: &'a str, b: &'a str) -> &'a str {
        let end = a
            .bytes()
            .zip(b.bytes())
            .take_while(|(x, y)| x.eq(y))
            .count();
        unsafe { a.get_unchecked(..end) }
    }

    #[inline]
    fn is_empty(instance: &str) -> bool {
        instance.is_empty()
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    #[inline]
    fn common<'a>(a: &'a str, b: &'a str) -> &'a str {
        let end = a
            .bytes()
            .rev()
            .zip(b.bytes().rev())
            .take_while(|(x, y)| x.eq(y))
            .count();
        let begin = a.len() - end;
        unsafe { a.get_unchecked(begin..) }
    }

    #[inline]
    fn is_empty(instance: &str) -> bool {
        instance.is_empty()
    }
}

pub struct GenericPrefix;
impl<T> Finder<[T]> for GenericPrefix
where
    T: Eq,
{
    #[inline]
    fn common<'a>(a: &'a [T], b: &'a [T]) -> &'a [T] {
        let end = a
            .into_iter()
            .zip(b.into_iter())
            .take_while(|(x, y)| x.eq(y))
            .count();
        unsafe { a.get_unchecked(..end) }
    }

    #[inline]
    fn is_empty(instance: &[T]) -> bool {
        instance.is_empty()
    }
}

pub struct GenericSuffix;
impl<T> Finder<[T]> for GenericSuffix
where
    T: Eq,
{
    #[inline]
    fn common<'a>(a: &'a [T], b: &'a [T]) -> &'a [T] {
        let end = a
            .into_iter()
            .rev()
            .zip(b.into_iter().rev())
            .take_while(|(x, y)| x.eq(y))
            .count();
        let begin = a.len() - end;
        unsafe { a.get_unchecked(begin..) }
    }

    #[inline]
    fn is_empty(instance: &[T]) -> bool {
        instance.is_empty()
    }
}
