use fastxfix::CommonStr;
use std::hint::black_box;
use ya_rand::*;

const COMMON: &str = "this is the common SHITE xD wowow";
const SIZES: [usize; 11] = [
    1 << 14,
    1 << 15,
    1 << 16,
    1 << 17,
    1 << 18,
    1 << 19,
    1 << 20,
    1 << 21,
    1 << 22,
    1 << 23,
    1 << 24,
];

fn main() {
    // Initialize rayon's global threadpool.
    rayon::join(|| black_box(()), || black_box(()));
    run_it_p();
    run_it_s();
}

fn gen_strings_s(size: usize) -> Vec<String> {
    let mut rng = new_rng();
    let mut vec = vec![String::with_capacity(128); size];
    vec.iter_mut().for_each(|v| {
        let s = rng.u64().to_string();
        v.push_str(&s);
        v.push_str(COMMON);
    });
    vec
}

fn run_it_s() {
    let strings = gen_strings_s(*SIZES.last().unwrap());
    for size in SIZES {
        let (suffix, time) = time(|| strings[..size].common_suffix());
        if suffix.is_some() && suffix.unwrap() == COMMON {
            print!("Successfully identified");
        } else {
            print!("Failed to identify");
        }
        println!(" common suffix for {} Strings in {:.4} seconds", size, time);
    }
    println!();
}

fn gen_strings_p(size: usize) -> Vec<String> {
    let mut rng = new_rng();
    let mut vec = vec![String::with_capacity(128); size];
    vec.iter_mut().for_each(|v| {
        let s = rng.u64().to_string();
        v.push_str(COMMON);
        v.push_str(&s);
    });
    vec
}

fn run_it_p() {
    let strings = gen_strings_p(*SIZES.last().unwrap());
    for size in SIZES {
        let (prefix, time) = time(|| strings[..size].common_prefix());
        if prefix.is_some() && prefix.unwrap() == COMMON {
            print!("Successfully identified");
        } else {
            print!("Failed to identify");
        }
        println!(" common prefix for {} Strings in {:.4} seconds", size, time);
    }
    println!();
}

fn time<F, T>(func: F) -> (T, f64)
where
    F: FnOnce() -> T,
{
    let start = std::time::Instant::now();
    let result = func();
    let time_delta = std::time::Instant::now()
        .duration_since(start)
        .as_secs_f64();
    (result, time_delta)
}
