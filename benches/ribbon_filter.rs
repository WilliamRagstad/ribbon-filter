use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
use std::time::Instant;

use ribbon_filter::{Mode, Params, RibbonBuilder};

use crate::common::ResultRow;

type H = BuildHasherDefault<DefaultHasher>;

pub fn measure(n: usize, w: usize, r: usize, seed: u64, q: usize) -> ResultRow {
    let params = Params::new(n * 4, w, r, Mode::Standard)
        .expect("valid params")
        .with_seed(seed)
        .with_retry_policy(4, 1)
        .expect("valid retry policy");

    let builder = RibbonBuilder::new(params, H::default()).expect("builder");
    let keys: Vec<u64> = (0..n as u64).collect();

    let start = Instant::now();
    let filter = builder.build(&keys).expect("build");
    let build_us = start.elapsed().as_micros();
    let bits_per_key = ((filter.params().m * filter.params().r) as f64) / (n as f64);
    let mut scratch = filter.new_scratch();

    let start = Instant::now();
    let mut hits = 0usize;
    for i in 0..q as u64 {
        if filter.contains_in(&(10_000_000 + i), &mut scratch) {
            hits += 1;
        }
    }
    let query_us = start.elapsed().as_micros();

    let _ = hits;
    ResultRow {
        name: "ribbon-filter",
        build_us,
        query_us,
        bits_per_key,
    }
}
