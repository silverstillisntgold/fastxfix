/*!
Contains the core [`Finder`] trait and concrete implementations for generic prefix/suffix lookup,
as well as specialized implementations for strings. Strings need to be handled seperately
because Rust strings use UTF-8 encoding, which requires special care to ensure correctness.

The approach I use to solve this issue is very simple. We initially treat the two strings as byte
slices and find the amount of bytes they have in common. We then treat this value as an index
into the first of the two byte slices, and feed it into a loop to adjust the index until it lies on a
valid char boundary. This adjusted index is then evaluated to determine if the slice it creates would
be non-empty, and if it is, that's our common prefix/suffix.

An easy optimization we can implement for this approach is to chunk the two byte slices so they can
fit into a 128-bit wide vector register (sse2/neon/simd128), and compare those chunks. Then we
can multiply the amount of equal chunks we find to the size of our chunks to determine how many
equivalent bytes the two strings have. After this, we just check byte-by-byte from where our
equal chunks ended to determine the total amount of equal consecutive bytes in the prefix/suffix,
and now we have an index which can be adjusted to the nearest char boundary and used for slicing.

# Safety

All implementations of [`Finder`] use `unsafe` when indexing the final slice/str being returned.
This indexing is safe because the index itself is directly derived from the minimum length of the two
slices/strs being compared.
*/

/// Equivalent to `__m128i::BITS` / `u8::BITS`. This allows the
/// string prefix/suffix methods to autovectorize their operations,
/// which provides a >50%+ speed increase on my machine.
///
/// Testing suggests that this doesn't scale all that well to larger
/// vector registers, even in examples containing relatively long
/// common prefixes/suffixes.
const CHUNK_SIZE: usize = 128 / 8;

trait EqCounter {
    fn count_eq(self) -> usize;
}

impl<T, U> EqCounter for T
where
    T: Iterator<Item = (U, U)>,
    U: Eq,
{
    /// Counts the amount of consecutive equal elements in an iterator of paired elements.
    #[inline]
    fn count_eq(self) -> usize {
        self.take_while(|(a, b)| a.eq(b)).count()
    }
}

pub trait Finder<T: ?Sized> {
    fn common<'a>(a: &'a T, b: &T) -> Option<&'a T>;
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

        let a_rem = a_bytes.iter().skip(end);
        let b_rem = b_bytes.iter().skip(end);
        end += a_rem.zip(b_rem).count_eq();

        while !a.is_char_boundary(end) {
            end -= 1;
        }
        match end > 0 {
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

        let a_rem = a_bytes.iter().rev().skip(end);
        let b_rem = b_bytes.iter().rev().skip(end);
        end += a_rem.zip(b_rem).count_eq();

        let mut begin = a.len() - end;
        while !a.is_char_boundary(begin) {
            begin += 1;
        }
        match begin < a.len() {
            true => Some(unsafe { a.get_unchecked(begin..) }),
            false => None,
        }
    }
}

pub struct GenericPrefix;
impl<T: Eq> Finder<[T]> for GenericPrefix {
    fn common<'a>(a: &'a [T], b: &[T]) -> Option<&'a [T]> {
        let a_iter = a.iter();
        let b_iter = b.iter();
        let end = a_iter.zip(b_iter).count_eq();
        match end > 0 {
            true => Some(unsafe { a.get_unchecked(..end) }),
            false => None,
        }
    }
}

pub struct GenericSuffix;
impl<T: Eq> Finder<[T]> for GenericSuffix {
    fn common<'a>(a: &'a [T], b: &[T]) -> Option<&'a [T]> {
        let a_iter = a.iter().rev();
        let b_iter = b.iter().rev();
        let end = a_iter.zip(b_iter).count_eq();
        let begin = a.len() - end;
        match begin < a.len() {
            true => Some(unsafe { a.get_unchecked(begin..) }),
            false => None,
        }
    }
}
