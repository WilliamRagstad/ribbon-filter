use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use clubcard::builder::{ApproximateRibbon, ClubcardBuilder, ExactRibbon};
use clubcard::{AsQuery, Equation, Filterable, Membership, Queryable};
use criterion::{BenchmarkId, Throughput, black_box};

use crate::common::{Group, QUERY_COUNT, SCENARIOS};

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

fn positives(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn negatives() -> Vec<u64> {
    (0..QUERY_COUNT as u64).map(|i| 10_000_000 + i).collect()
}

fn make_filter(
    positive_values: &[u64],
    negative_values: &[u64],
) -> clubcard::Clubcard<CLUBCARD_WORDS, (), ()> {
    let mut builder = ClubcardBuilder::<CLUBCARD_WORDS, ClubcardItem>::new();

    let mut approx_builder = builder.new_approx_builder(&[]);
    approx_builder.set_universe_size(positive_values.len() + negative_values.len());
    for &key in positive_values {
        approx_builder.insert(ClubcardItem::new(key, true));
    }
    builder.collect_approx_ribbons(vec![ApproximateRibbon::from(approx_builder)]);

    let mut exact_builder = builder.new_exact_builder(&[]);
    for &key in positive_values {
        exact_builder.insert(ClubcardItem::new(key, true));
    }
    for &value in negative_values {
        exact_builder.insert(ClubcardItem::new(value, false));
    }
    builder.collect_exact_ribbons(vec![ExactRibbon::from(exact_builder)]);

    builder.build::<ClubcardItem>((), ())
}

pub fn bench_build(group: &mut Group<'_>) {
    let negative_values = negatives();

    for scenario in SCENARIOS {
        let positive_values = positives(scenario.n);
        let id = BenchmarkId::new("clubcard", scenario.id());
        group.throughput(Throughput::Elements((scenario.n + QUERY_COUNT) as u64));
        group.bench_with_input(id, &positive_values, |b, positives| {
            b.iter(|| {
                black_box(make_filter(positives, &negative_values));
            });
        });
    }
}

pub fn bench_query(group: &mut Group<'_>) {
    let query_values = negatives();

    for scenario in SCENARIOS {
        let positive_values = positives(scenario.n);
        let filter = make_filter(&positive_values, &query_values);
        let id = BenchmarkId::new("clubcard", scenario.id());
        group.throughput(Throughput::Elements(QUERY_COUNT as u64));

        group.bench_with_input(id, &query_values, |b, values| {
            b.iter(|| {
                let mut hits = 0usize;
                for &value in values {
                    let member = matches!(
                        filter.contains(&ClubcardItem::new(value, false)),
                        Membership::Member
                    );
                    if member {
                        hits += 1;
                    }
                }
                black_box(hits);
            });
        });
    }
}
