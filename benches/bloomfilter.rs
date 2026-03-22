use bloomfilter::Bloom;
use criterion::{BenchmarkId, Throughput, black_box};

use crate::common::{FP_RATE, Group, QUERY_COUNT, SCENARIOS};

fn keys(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn queries() -> Vec<u64> {
    (0..QUERY_COUNT as u64).map(|i| 10_000_000 + i).collect()
}

pub fn bench_build(group: &mut Group<'_>) {
    for scenario in SCENARIOS {
        let keys = keys(scenario.n);
        let id = BenchmarkId::new("bloomfilter", scenario.id());
        group.throughput(Throughput::Elements(scenario.n as u64));
        group.bench_with_input(id, &keys, |b, keys| {
            b.iter(|| {
                let mut filter =
                    Bloom::new_for_fp_rate(keys.len(), FP_RATE).expect("valid bloomfilter params");
                for key in keys {
                    filter.set(key);
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
        let mut filter =
            Bloom::new_for_fp_rate(keys.len(), FP_RATE).expect("valid bloomfilter params");
        for key in keys {
            filter.set(&key);
        }

        let id = BenchmarkId::new("bloomfilter", scenario.id());
        group.throughput(Throughput::Elements(QUERY_COUNT as u64));
        group.bench_with_input(id, &query_values, |b, values| {
            b.iter(|| {
                let mut hits = 0usize;
                for value in values {
                    if filter.check(value) {
                        hits += 1;
                    }
                }
                black_box(hits);
            });
        });
    }
}
