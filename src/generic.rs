use crate::{
    finder::Finder,
    joiner::{Joiner, Parallel, Sequential},
    PAR_THRESHOLD,
};

pub fn find_common<F, T, U>(slice: &[T]) -> Option<&U>
where
    F: Finder<U>,
    T: AsRef<U> + Sync,
    U: ?Sized + Sync,
{
    match slice.is_empty() {
        false => {
            let result = find_common_recurse::<F, Parallel, _, _>(slice);
            match F::is_empty(result) {
                false => Some(result),
                true => None,
            }
        }
        true => None,
    }
}

fn find_common_recurse<F, J, T, U>(slice: &[T]) -> &U
where
    F: Finder<U>,
    J: Joiner,
    T: AsRef<U> + Sync,
    U: ?Sized + Sync,
{
    // SAFETY: We never start recursing when `slice` is empty.
    unsafe {
        core::hint::assert_unchecked(!slice.is_empty());
    }

    if !J::IS_PARALLEL && slice.len() < 3 {
        if slice.len() == 1 {
            return slice[0].as_ref();
        };
        return F::common(slice[0].as_ref(), slice[1].as_ref());
    }

    // Break off parallel recursion into sequential recursion, but only
    // when currently recursing in parallel.
    // Since `IS_PARALLEL` is a constant, rustc is able to strip it
    // out of both generated variants.
    if J::IS_PARALLEL && slice.len() <= PAR_THRESHOLD {
        return find_common_recurse::<F, Sequential, _, _>(slice);
    }

    let (left, right) = slice.split_at(slice.len() / 2);
    let (left, right) = J::join(
        || find_common_recurse::<F, J, _, _>(left),
        || find_common_recurse::<F, J, _, _>(right),
    );

    F::common(left, right)
}
