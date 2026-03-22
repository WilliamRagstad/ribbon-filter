use std::time::Instant;

use fastbloom_rs::{FilterBuilder as FastBloomBuilder, Membership};

use crate::common::{ResultRow, FP_RATE};

pub fn measure(n: usize, q: usize) -> ResultRow {
    let keys: Vec<u64> = (0..n as u64).collect();
    let mut builder = FastBloomBuilder::new(n as u64, FP_RATE);

    let build_start = Instant::now();
    let mut filter = builder.build_bloom_filter();
    for &key in &keys {
        filter.add(&key.to_le_bytes());
    }
    let build_us = build_start.elapsed().as_micros();

    let cfg = filter.config();
    let bits_per_key = (cfg.size as f64) / (n as f64);

    let query_start = Instant::now();
    let mut hits = 0usize;
    for i in 0..q as u64 {
        if filter.contains(&(10_000_000 + i).to_le_bytes()) {
            hits += 1;
        }
    }
    let query_us = query_start.elapsed().as_micros();

    let _ = hits;
    ResultRow {
        name: "fastbloom-rs",
        build_us,
        query_us,
        bits_per_key,
    }
}
