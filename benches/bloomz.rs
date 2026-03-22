use bloomz::BloomFilter as BloomzFilter;
use criterion::{BenchmarkId, Throughput, black_box};

use crate::common::{FP_RATE, Group, QUERY_COUNT, SCENARIOS};

fn keys(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn queries() -> Vec<u64> {
    (0..QUERY_COUNT as u64).map(|i| 10_000_000 + i).collect()
}

fn make_filter(keys: &[u64]) -> BloomzFilter {
    let mut filter = BloomzFilter::new_for_capacity(keys.len(), FP_RATE);
    for key in keys {
        filter.insert(key);
    }
    filter
}

pub fn bench_build(group: &mut Group<'_>) {
    for scenario in SCENARIOS {
        let keys = keys(scenario.n);
        let id = BenchmarkId::new("bloomz", scenario.id());
        group.throughput(Throughput::Elements(scenario.n as u64));
        group.bench_with_input(id, &keys, |b, keys| {
            b.iter(|| {
                black_box(make_filter(keys));
            });
        });
    }
}

pub fn bench_query(group: &mut Group<'_>) {
    let query_values = queries();

    for scenario in SCENARIOS {
        let keys = keys(scenario.n);
        let filter = make_filter(&keys);

        let id = BenchmarkId::new("bloomz", scenario.id());
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
