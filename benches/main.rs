#![allow(clippy::print_stdout)]

mod bloomfilter;
mod bloomz;
mod clubcard;
mod common;
mod fastbloom;
mod ribbonfilter;

use common::QUERY_COUNT;

fn main() {
    let scenarios = [
        (10_000usize, 16usize, 8usize, 42u64),
        (100_000usize, 16usize, 8usize, 42u64),
        (100_000usize, 96usize, 10usize, 777u64),
    ];

    println!("impl,scenario,build_us,query_us,bits_per_key");
    for (n, w, r, seed) in scenarios {
        for row in [
            ribbonfilter::measure(n, w, r, seed, QUERY_COUNT),
            fastbloom::measure(n, QUERY_COUNT),
            bloomfilter::measure(n, QUERY_COUNT),
            bloomz::measure(n, QUERY_COUNT),
            clubcard::measure(n, QUERY_COUNT),
        ] {
            println!(
                "{},n={n};w={w};r={r},{},{},{:.4}",
                row.name, row.build_us, row.query_us, row.bits_per_key
            );
        }
    }
}
