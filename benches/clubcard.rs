use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use clubcard::builder::{ApproximateRibbon, ClubcardBuilder, ExactRibbon};
use clubcard::{ApproximateSizeOf, AsQuery, Equation, Filterable, Queryable};

use crate::common::ResultRow;

const CLUBCARD_WORDS: usize = 4;

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

pub fn measure(n: usize, q: usize) -> ResultRow {
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
