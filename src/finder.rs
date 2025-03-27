/// `__m128i::BITS` / `u8::BITS`
const CHUNK_SIZE: usize = 128 / 8;

pub trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &T) -> Option<&'a T>;
}

trait EqCounter {
    fn count_eq(self) -> usize;
}

impl<T, U> EqCounter for T
where
    T: Iterator<Item = (U, U)>,
    U: Eq,
{
    #[inline(always)]
    fn count_eq(self) -> usize {
        self.take_while(|(a, b)| a.eq(b)).count()
    }
}

pub struct StringPrefix;
impl Finder<str> for StringPrefix {
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();

        let a_chunks = a_bytes.chunks_exact(CHUNK_SIZE);
        let b_chunks = b_bytes.chunks_exact(CHUNK_SIZE);
        let mut end = a_chunks.zip(b_chunks).count_eq() * CHUNK_SIZE;

        let a_rem = a_bytes.into_iter().skip(end);
        let b_rem = b_bytes.into_iter().skip(end);
        end += a_rem.zip(b_rem).count_eq();

        while !a.is_char_boundary(end) {
            end -= 1;
        }
        match end != 0 {
            true => Some(unsafe { a.get_unchecked(..end) }),
            false => None,
        }
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();

        let a_chunks = a_bytes.rchunks_exact(CHUNK_SIZE);
        let b_chunks = b_bytes.rchunks_exact(CHUNK_SIZE);
        let mut end = a_chunks.zip(b_chunks).count_eq() * CHUNK_SIZE;

        let a_rem = a_bytes.into_iter().rev().skip(end);
        let b_rem = b_bytes.into_iter().rev().skip(end);
        end += a_rem.zip(b_rem).count_eq();

        let mut begin = a.len() - end;
        while !a.is_char_boundary(begin) {
            begin += 1;
        }
        match begin != a.len() {
            true => Some(unsafe { a.get_unchecked(begin..) }),
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
