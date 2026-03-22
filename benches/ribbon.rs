#![allow(clippy::print_stdout)]

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use bloomfilter::Bloom;
use bloomz::BloomFilter as BloomzFilter;
use clubcard::builder::{ApproximateRibbon, ClubcardBuilder, ExactRibbon};
use clubcard::{ApproximateSizeOf, AsQuery, Equation, Filterable, Queryable};
use fastbloom_rs::{FilterBuilder as FastBloomBuilder, Membership};
use ribbon_filter::{Mode, Params, RibbonBuilder};

type H = BuildHasherDefault<DefaultHasher>;

const FP_RATE: f64 = 0.01;
const QUERY_COUNT: usize = 1_000_000;
const CLUBCARD_WORDS: usize = 4;

struct ResultRow {
    name: &'static str,
    build_us: u128,
    query_us: u128,
    bits_per_key: f64,
}

#[derive(Clone)]
struct ClubcardItem {
    key: u64,
    include: bool,
    key_bytes: [u8; 8],
}

impl ClubcardItem {
    fn new(key: u64, include: bool) -> Self {
        Self {
            key,
            include,
            key_bytes: key.to_le_bytes(),
        }
    }
}

fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

impl AsQuery<CLUBCARD_WORDS> for ClubcardItem {
    fn as_query(&self, m: usize) -> Equation<CLUBCARD_WORDS> {
        let mut hasher = DefaultHasher::new();
        self.key.hash(&mut hasher);
        let base = hasher.finish();

        let mut coeffs = [0u64; CLUBCARD_WORDS];
        let mut state = base;
        for coeff in &mut coeffs {
            state = mix64(state);
            *coeff = state;
        }
        coeffs[0] |= 1;

        let s = if m == 0 {
            0
        } else {
            (mix64(base) as usize) % m
        };
        Equation::homogeneous(s, coeffs)
    }

    fn block(&self) -> &[u8] {
        &[]
    }

    fn discriminant(&self) -> &[u8] {
        &self.key_bytes
    }
}

impl Filterable<CLUBCARD_WORDS> for ClubcardItem {
    fn included(&self) -> bool {
        self.include
    }
}

impl Queryable<CLUBCARD_WORDS> for ClubcardItem {
    type UniverseMetadata = ();
    type PartitionMetadata = ();

    fn in_universe(&self, _meta: &Self::UniverseMetadata) -> bool {
        true
    }
}

fn measure_ribbon(n: usize, w: usize, r: usize, seed: u64, q: usize) -> ResultRow {
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

fn measure_clubcard(n: usize, q: usize) -> ResultRow {
    let keys: Vec<u64> = (0..n as u64).collect();
    let query_values: Vec<u64> = (0..q as u64).map(|i| 10_000_000 + i).collect();

    let build_start = Instant::now();

    let mut builder = ClubcardBuilder::<CLUBCARD_WORDS, ClubcardItem>::new();
    let mut approx_builder = builder.new_approx_builder(&[]);
    approx_builder.set_universe_size(n + q);
    for &key in &keys {
        approx_builder.insert(ClubcardItem::new(key, true));
    }
    builder.collect_approx_ribbons(vec![ApproximateRibbon::from(approx_builder)]);

    let mut exact_builder = builder.new_exact_builder(&[]);
    for &key in &keys {
        exact_builder.insert(ClubcardItem::new(key, true));
    }
    for &value in &query_values {
        exact_builder.insert(ClubcardItem::new(value, false));
    }
    builder.collect_exact_ribbons(vec![ExactRibbon::from(exact_builder)]);

    let filter = builder.build::<ClubcardItem>((), ());
    let build_us = build_start.elapsed().as_micros();
    let bits_per_key = ((filter.approximate_size_of() * 8) as f64) / (n as f64);

    let query_start = Instant::now();
    let mut hits = 0usize;
    for &value in &query_values {
        if filter.unchecked_contains(&ClubcardItem::new(value, false)) {
            hits += 1;
        }
    }
    let query_us = query_start.elapsed().as_micros();

    let _ = hits;
    ResultRow {
        name: "clubcard",
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
        for row in [
            measure_ribbon(n, w, r, seed, QUERY_COUNT),
            measure_fastbloom(n, QUERY_COUNT),
            measure_bloomfilter(n, QUERY_COUNT),
            measure_bloomz(n, QUERY_COUNT),
            measure_clubcard(n, QUERY_COUNT),
        ] {
            println!(
                "{},n={n};w={w};r={r},{},{},{:.4}",
                row.name, row.build_us, row.query_us, row.bits_per_key
            );
        }
    }
}
