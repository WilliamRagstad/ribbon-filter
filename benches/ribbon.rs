#![allow(clippy::print_stdout)]

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
use std::time::Instant;

use bloomfilter::Bloom;
use bloomz::BloomFilter as BloomzFilter;
use fastbloom_rs::{FilterBuilder as FastBloomBuilder, Membership};
use ribbon_filter::{Mode, Params, RibbonBuilder};

type H = BuildHasherDefault<DefaultHasher>;

const FP_RATE: f64 = 0.01;
const QUERY_COUNT: usize = 1_000_000;

struct ResultRow {
    name: &'static str,
    build_us: u128,
    query_us: u128,
    bits_per_key: f64,
}

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

fn measure_fastbloom(n: usize, q: usize) -> ResultRow {
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

fn measure_bloomfilter(n: usize, q: usize) -> ResultRow {
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

fn bloomz_bits(filter: &BloomzFilter) -> usize {
    let bytes = filter.to_bytes();
    let m_offset = bytes.len() - 12;
    let mut m_arr = [0u8; 8];
    m_arr.copy_from_slice(&bytes[m_offset..m_offset + 8]);
    u64::from_le_bytes(m_arr) as usize
}

fn measure_bloomz(n: usize, q: usize) -> ResultRow {
    let keys: Vec<u64> = (0..n as u64).collect();

    let build_start = Instant::now();
    let mut filter = BloomzFilter::new_for_capacity(n, FP_RATE);
    for &key in &keys {
        filter.insert(&key);
    }
    let build_us = build_start.elapsed().as_micros();

    let bits_per_key = (bloomz_bits(&filter) as f64) / (n as f64);

    let query_start = Instant::now();
    let mut hits = 0usize;
    for i in 0..q as u64 {
        if filter.contains(&(10_000_000 + i)) {
            hits += 1;
        }
    }
    let query_us = query_start.elapsed().as_micros();

    let _ = hits;
    ResultRow {
        name: "bloomz",
        build_us,
        query_us,
        bits_per_key,
    }
}

fn main() {
    let scenarios = [
        (10_000usize, 16usize, 8usize, 42u64),
        (100_000usize, 16usize, 8usize, 42u64),
        (100_000usize, 96usize, 10usize, 777u64),
    ];

    println!("impl,scenario,build_us,query_us,bits_per_key");
    for (n, w, r, seed) in scenarios {
        let (build_us, bits_total) = measure_build(n, w, r, seed);
        let query_us = measure_query(n, w, r, seed, QUERY_COUNT);
        let bits_per_key = (bits_total as f64) / (n as f64);

        println!("ribbon-filter,n={n};w={w};r={r},{build_us},{query_us},{bits_per_key:.4}");

        for row in [
            measure_fastbloom(n, QUERY_COUNT),
            measure_bloomfilter(n, QUERY_COUNT),
            measure_bloomz(n, QUERY_COUNT),
        ] {
            println!(
                "{},n={n};w={w};r={r},{},{},{:.4}",
                row.name, row.build_us, row.query_us, row.bits_per_key
            );
        }
    }
}
