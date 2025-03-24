#![allow(unused)]

use fastxfix::*;
use rayon::prelude::*;
use ya_rand::*;

const COMMON: &str = "愛 This is the common SHITE xD 愛";
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
    rayon::join(|| (), || ());
    println!("starting program");
    run_it_p();
    run_it_s();
    for _ in 0..64 {
        let prefix = random_string::<64, 64>(&mut new_rng());
        let suffix = prefix.clone();
        let mut string_list = random_strings(*SIZES.first().unwrap());
        append_at_front(&mut string_list, &prefix);
        append_at_rear(&mut string_list, &suffix);

        let t1 = string_list.common_prefix().unwrap();
        if prefix == t1 {
            //println!("Successfully identified prefix!");
        } else {
            println!("FAIL prefix!");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let t2 = string_list.common_suffix().unwrap();
        if suffix == t2 {
            //println!("Successfully identified suffix!");
        } else {
            println!("FAIL suffix!");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    println!("SUCCESS");
    println!();
}

fn append_at_front(strings: &mut [String], append: &str) {
    strings.into_par_iter().for_each(|s| {
        s.insert_str(0, append);
    });
}

fn append_at_rear(strings: &mut [String], append: &str) {
    strings.into_par_iter().for_each(|s| {
        s.push_str(append);
    });
}

fn random_strings(string_count: usize) -> Vec<String> {
    assert!(string_count >= 2);
    let mut rng = new_rng();
    let mut strings = Vec::with_capacity(string_count);
    while strings.len() < string_count {
        let s = random_string::<16, 16>(&mut rng);
        strings.push(s);
    }
    strings
}

fn random_string<const MIN: i64, const MAX: i64>(rng: &mut ShiroRng) -> String {
    const {
        assert!(MAX >= MIN);
    }
    let cap = rng.range_inclusive(MIN, MAX) as usize;
    let mut chars = String::with_capacity(cap);
    while chars.len() < cap {
        // 2^21 is the lowest power of two above max char value
        match char::from_u32(rng.bits(21) as u32) {
            Some(c) => chars.push(c),
            None => {}
        }
    }
    chars
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
    let res = func();
    let time_delta = std::time::Instant::now()
        .duration_since(start)
        .as_secs_f64();
    (res, time_delta)
}
