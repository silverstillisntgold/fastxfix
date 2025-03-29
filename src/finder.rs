/*!
Contains the core `Finder` trait and concrete implementation for generic prefix/suffix lookup,
as well as a specialized implementation for strings. Strings need to be handled seperately
because Rust strings use UTF-8 encoding, so just comparing byte by byte wouldn't work correctly
for all cases. There are many instances where such an approach might exhibit correct behavior
(strings which contain only ASCII values is an obvious one), but such an approach may quickly
devolve into UB when encountering a string where byte-equality ends half-way through a valid char.

The approach I use to solve this issue is very simple. We treat the two strings as byte slices,
then find the amount of bytes they have in common. We then treat this value as an index
into the `a` byte slice, and feed it into a loop to adjust the index until it lies on a valid
char boundary. This adjusted index is then evaluated to determine if the slice it creates would
be non-empty, and if it is, that's our common prefix/suffix.

An easy optimization for the string implementation is to chunk the two byte slices so they can
fit into a 128-bit wide vector registor (sse2/neon/simd128), and compare those chunks. Then we
can multiply the amount of equal chunks we found to the size of our chunks to determine how many
equivalent bytes the two strings have. After this, we need to check byte-by-byte from where our
chunks ended to find the final amount of equal bytes in the prefix/suffix.
In 40 fucking years, when Rust gets specialization, it should be possible to do something similar
with specialization(s) for the generic `Finder` implementations.
*/

/// Equivalent to `__m128i::BITS` / `u8::BITS`. This allows the
/// string prefix/suffix methods to autovectorize their operation,
/// providing a 50%+ speed increase (on my machine).
///
/// Testing suggests that this doesn't scale all that well to larger
/// vector registers, even in collections containing relatively long
/// prefixes/suffixes.
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
        let mut end = a_chunks.zip(b_chunks).count_eq();
        end *= CHUNK_SIZE;

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
        let mut end = a_chunks.zip(b_chunks).count_eq();
        end *= CHUNK_SIZE;

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
        let begin = a.len() - end;
        match begin != a.len() {
            true => Some(unsafe { a.get_unchecked(begin..) }),
            false => None,
        }
    }
}
