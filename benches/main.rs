mod bloomfilter;
mod bloomz;
mod clubcard;
mod common;
mod fastbloom;
mod ribbonfilter;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("build");
    group.sample_size(50);

    ribbonfilter::bench_build(&mut group);
    fastbloom::bench_build(&mut group);
    bloomfilter::bench_build(&mut group);
    bloomz::bench_build(&mut group);
    clubcard::bench_build(&mut group);

    group.finish();
}

fn bench_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("query");
    group.sample_size(50);

    ribbonfilter::bench_query(&mut group);
    fastbloom::bench_query(&mut group);
    bloomfilter::bench_query(&mut group);
    bloomz::bench_query(&mut group);
    clubcard::bench_query(&mut group);

    group.finish();
}

criterion_group!(benches, bench_build, bench_query);
criterion_main!(benches);
