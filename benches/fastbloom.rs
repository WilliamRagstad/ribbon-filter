use criterion::{BenchmarkId, Throughput, black_box};
use fastbloom_rs::{FilterBuilder as FastBloomBuilder, Membership};

use crate::common::{FP_RATE, Group, QUERY_COUNT, SCENARIOS};

fn keys(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn queries() -> Vec<[u8; 8]> {
    (0..QUERY_COUNT as u64)
        .map(|i| (10_000_000 + i).to_le_bytes())
        .collect()
}

pub fn bench_build(group: &mut Group<'_>) {
    for scenario in SCENARIOS {
        let keys = keys(scenario.n);
        let id = BenchmarkId::new("fastbloom-rs", scenario.id());
        group.throughput(Throughput::Elements(scenario.n as u64));
        group.bench_with_input(id, &keys, |b, keys| {
            b.iter(|| {
                let mut builder = FastBloomBuilder::new(keys.len() as u64, FP_RATE);
                let mut filter = builder.build_bloom_filter();
                for key in keys {
                    filter.add(&key.to_le_bytes());
                }
                black_box(filter);
            });
        });
    }
}

pub fn bench_query(group: &mut Group<'_>) {
    let query_values = queries();

    for scenario in SCENARIOS {
        let keys = keys(scenario.n);
        let mut builder = FastBloomBuilder::new(keys.len() as u64, FP_RATE);
        let mut filter = builder.build_bloom_filter();
        for key in keys {
            filter.add(&key.to_le_bytes());
        }

        let id = BenchmarkId::new("fastbloom-rs", scenario.id());
        group.throughput(Throughput::Elements(QUERY_COUNT as u64));
        group.bench_with_input(id, &query_values, |b, values| {
            b.iter(|| {
                let mut hits = 0usize;
                for value in values {
                    if filter.contains(value) {
                        hits += 1;
                    }
                }
                black_box(hits);
            });
        });
    }
}
