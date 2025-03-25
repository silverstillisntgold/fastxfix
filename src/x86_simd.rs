use crate::finder::finalize_prefix;
use crate::finder::finalize_suffix;
use crate::Finder;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

const AVX2_STEP_SIZE: isize = size_of::<__m256i>() as isize;
const SSE2_STEP_SIZE: isize = size_of::<__m128i>() as isize;

#[inline(always)]
unsafe fn avx2_mask(a_ptr: *const u8, b_ptr: *const u8, i: isize) -> u32 {
    let a_chunk = _mm256_loadu_si256(a_ptr.add(i as usize).cast());
    let b_chunk = _mm256_loadu_si256(b_ptr.add(i as usize).cast());
    let byte_cmp = _mm256_cmpeq_epi8(a_chunk, b_chunk);
    _mm256_movemask_epi8(byte_cmp) as u32
}

#[inline(always)]
unsafe fn sse2_mask(a_ptr: *const u8, b_ptr: *const u8, i: isize) -> u32 {
    let a_chunk = _mm_loadu_si128(a_ptr.add(i as usize).cast());
    let b_chunk = _mm_loadu_si128(b_ptr.add(i as usize).cast());
    let byte_cmp = _mm_cmpeq_epi8(a_chunk, b_chunk);
    _mm_movemask_epi8(byte_cmp) as u32
}

pub struct StringPrefix;
impl Finder<str> for StringPrefix {
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let len = a.len().min(b.len()) as isize;
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let a_ptr = a_bytes.as_ptr();
        let b_ptr = b_bytes.as_ptr();
        let mut i = 0 as isize;
        #[cfg(target_feature = "avx2")]
        {
            while i.wrapping_add(AVX2_STEP_SIZE) <= len {
                let cmp_mask = unsafe { avx2_mask(a_ptr, b_ptr, i) };
                match cmp_mask == 0xFFFFFFFF {
                    true => {
                        i = i.wrapping_add(AVX2_STEP_SIZE);
                    }
                    false => {
                        i = i.wrapping_add(cmp_mask.trailing_ones() as isize);
                        return finalize_prefix(a, i);
                    }
                }
            }
        }
        while i.wrapping_add(SSE2_STEP_SIZE) <= len {
            let cmp_mask = unsafe { sse2_mask(a_ptr, b_ptr, i) };
            match cmp_mask == 0xFFFF {
                true => {
                    i = i.wrapping_add(SSE2_STEP_SIZE);
                }
                false => {
                    i = i.wrapping_add(cmp_mask.trailing_ones() as isize);
                    return finalize_prefix(a, i);
                }
            }
        }
        while i < len && a_bytes[i as usize] == b_bytes[i as usize] {
            i = i.wrapping_add(1);
        }

        finalize_prefix(a, i)
    }
}

pub struct StringSuffix;
impl Finder<str> for StringSuffix {
    fn common<'a>(a: &'a str, b: &str) -> Option<&'a str> {
        let len = a.len().min(b.len());
        let a_bytes = &a.as_bytes()[(a.len() - len)..];
        let b_bytes = &b.as_bytes()[(b.len() - len)..];
        let a_newlen = unsafe { str::from_utf8_unchecked(a_bytes) };
        let a_ptr = a_bytes.as_ptr();
        let b_ptr = b_bytes.as_ptr();
        let mut i = len as isize;
        #[cfg(target_feature = "avx2")]
        {
            while i.wrapping_sub(AVX2_STEP_SIZE) >= 0 {
                let cmp_mask = unsafe { avx2_mask(a_ptr, b_ptr, i.wrapping_sub(AVX2_STEP_SIZE)) }
                    .reverse_bits();
                match cmp_mask == 0xFFFFFFFF {
                    true => {
                        i = i.wrapping_sub(AVX2_STEP_SIZE);
                    }
                    false => {
                        i = i.wrapping_sub(cmp_mask.trailing_ones() as isize);
                        return finalize_suffix(a_newlen, i);
                    }
                }
            }
        }
        while i.wrapping_sub(SSE2_STEP_SIZE) >= 0 {
            let cmp_mask = unsafe { sse2_mask(a_ptr, b_ptr, i.wrapping_sub(SSE2_STEP_SIZE)) }
                .reverse_bits()
                >> (u32::BITS as isize - SSE2_STEP_SIZE);
            match cmp_mask == 0xFFFF {
                true => {
                    i = i.wrapping_sub(SSE2_STEP_SIZE);
                }
                false => {
                    i = i.wrapping_sub(cmp_mask.trailing_ones() as isize);
                    return finalize_suffix(a_newlen, i);
                }
            }
        }
        while i > 0 && a_bytes[i as usize - 1] == b_bytes[i as usize - 1] {
            i = i.wrapping_sub(1);
        }
        finalize_suffix(a_newlen, i)
    }
}
