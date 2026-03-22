use std::time::Instant;

use bloomfilter::Bloom;

use crate::common::{FP_RATE, ResultRow};

pub fn measure(n: usize, q: usize) -> ResultRow {
    let keys: Vec<u64> = (0..n as u64).collect();

    let build_start = Instant::now();
    let mut filter = Bloom::new_for_fp_rate(n, FP_RATE).expect("valid bloomfilter params");
    for &key in &keys {
        filter.set(&key);
    }
    let build_us = build_start.elapsed().as_micros();

    let bits_per_key = (filter.len() as f64) / (n as f64);

    let query_start = Instant::now();
    let mut hits = 0usize;
    for i in 0..q as u64 {
        if filter.check(&(10_000_000 + i)) {
            hits += 1;
        }
    }
    let query_us = query_start.elapsed().as_micros();

    let _ = hits;
    ResultRow {
        name: "bloomfilter",
        build_us,
        query_us,
        bits_per_key,
    }
}
