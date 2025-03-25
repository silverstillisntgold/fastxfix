use crate::finder::finalize_prefix;
use crate::finder::finalize_suffix;
use crate::Finder;
use core::arch::aarch64::*;

const NEON_STEP_SIZE: isize = size_of::<uint8x16_t>() as isize;
const WEIGHTS: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];

#[inline(always)]
unsafe fn neon_mask(a_ptr: *const u8, b_ptr: *const u8, i: isize) -> u32 {
    let a_chunk = vld1q_u8(a_ptr.add(i as usize));
    let b_chunk = vld1q_u8(b_ptr.add(i as usize));
    let byte_cmp = vceqq_u8(a_chunk, b_chunk);
    let bits = vshrq_n_u8(byte_cmp, 7);
    let weights = vld1q_u8(WEIGHTS.as_ptr());
    let weighted = vmulq_u8(bits, weights);
    let low = vget_low_u8(weighted);
    let high = vget_high_u8(weighted);
    let low_sum = vaddv_u8(low) as u32;
    let high_sum = vaddv_u8(high) as u32;
    (high_sum << 8) | low_sum
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
        while i.wrapping_add(NEON_STEP_SIZE) <= len {
            let cmp_mask = unsafe { neon_mask(a_ptr, b_ptr, i) };
            match cmp_mask == 0xFFFF {
                true => {
                    i = i.wrapping_add(NEON_STEP_SIZE);
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
        while i.wrapping_sub(NEON_STEP_SIZE) >= 0 {
            let cmp_mask = unsafe { neon_mask(a_ptr, b_ptr, i.wrapping_sub(NEON_STEP_SIZE)) }
                .reverse_bits()
                >> (u32::BITS as isize - NEON_STEP_SIZE);
            match cmp_mask == 0xFFFF {
                true => {
                    i = i.wrapping_sub(NEON_STEP_SIZE);
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
