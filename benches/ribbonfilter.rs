use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

use criterion::{BenchmarkId, Throughput, black_box};
use ribbon_filter::{Mode, Params, RibbonBuilder};

use crate::common::{Group, QUERY_COUNT, SCENARIOS};

type H = BuildHasherDefault<DefaultHasher>;

fn keys(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn queries() -> Vec<u64> {
    (0..QUERY_COUNT as u64).map(|i| 10_000_000 + i).collect()
}

fn make_filter(n: usize, w: usize, r: usize, seed: u64) -> ribbon_filter::RibbonFilter<H> {
    let params = Params::new(n * 4, w, r, Mode::Standard)
        .expect("valid params")
        .with_seed(seed)
        .with_retry_policy(4, 1)
        .expect("valid retry policy");
    let builder = RibbonBuilder::new(params, H::default()).expect("builder");
    builder.build(&keys(n)).expect("build")
}

pub fn bench_build(group: &mut Group<'_>) {
    for scenario in SCENARIOS {
        let id = BenchmarkId::new("ribbon-filter", scenario.id());
        group.throughput(Throughput::Elements(scenario.n as u64));
        group.bench_with_input(id, &scenario, |b, s| {
            b.iter(|| {
                black_box(make_filter(s.n, s.w, s.r, s.seed));
            });
        });
    }
}

pub fn bench_query(group: &mut Group<'_>) {
    let query_values = queries();

    for scenario in SCENARIOS {
        let filter = make_filter(scenario.n, scenario.w, scenario.r, scenario.seed);
        let mut scratch = filter.new_scratch();
        let id = BenchmarkId::new("ribbon-filter", scenario.id());
        group.throughput(Throughput::Elements(QUERY_COUNT as u64));

        group.bench_with_input(id, &query_values, |b, values| {
            b.iter(|| {
                let mut hits = 0usize;
                for &value in values {
                    if filter.contains_in(&value, &mut scratch) {
                        hits += 1;
                    }
                }
                black_box(hits);
            });
        });
    }
}
