use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
use std::time::Instant;

use ribbon_filter::{Mode, Params, RibbonBuilder};

type H = BuildHasherDefault<DefaultHasher>;

fn measure_build(n: usize, w: usize, r: usize, seed: u64) -> (u128, usize) {
    let params = Params::new(n * 4, w, r, Mode::Standard)
        .expect("valid params")
        .with_seed(seed)
        .with_retry_policy(4, 1)
        .expect("valid retry policy");

    let builder = RibbonBuilder::new(params, H::default()).expect("builder");
    let keys: Vec<u64> = (0..n as u64).collect();

    let start = Instant::now();
    let filter = builder.build(&keys).expect("build");
    let elapsed = start.elapsed().as_micros();

    let bits = filter.params().m * filter.params().r;
    (elapsed, bits)
}

fn measure_query(n: usize, w: usize, r: usize, seed: u64, q: usize) -> u128 {
    let params = Params::new(n * 4, w, r, Mode::Standard)
        .expect("valid params")
        .with_seed(seed)
        .with_retry_policy(4, 1)
        .expect("valid retry policy");

    let builder = RibbonBuilder::new(params, H::default()).expect("builder");
    let keys: Vec<u64> = (0..n as u64).collect();
    let filter = builder.build(&keys).expect("build");
    let mut scratch = filter.new_scratch();

    let start = Instant::now();
    let mut hits = 0usize;
    for i in 0..q as u64 {
        if filter.contains_in(&(10_000_000 + i), &mut scratch) {
            hits += 1;
        }
    }
    let elapsed = start.elapsed().as_micros();

    let _ = hits;
    elapsed
}

fn main() {
    let scenarios = [
        (10_000usize, 16usize, 8usize, 42u64),
        (100_000usize, 16usize, 8usize, 42u64),
        (100_000usize, 96usize, 10usize, 777u64),
    ];

    println!("scenario,build_us,query_us,bits_per_key");
    for (n, w, r, seed) in scenarios {
        let (build_us, bits_total) = measure_build(n, w, r, seed);
        let query_us = measure_query(n, w, r, seed, 1_000_000);
        let bits_per_key = (bits_total as f64) / (n as f64);
        println!("n={n};w={w};r={r},{build_us},{query_us},{bits_per_key:.4}");
    }
}
