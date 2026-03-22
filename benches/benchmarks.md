# Benchmark Notes

This file keeps the last baseline captured by the legacy one-shot `Instant` harness.

The benchmark suite now uses Criterion (`cargo bench --bench main`) with statistical sampling,
confidence intervals, and grouped build/query benchmarks.

For current benchmark output and regressions:

- run locally and inspect `target/criterion/`
- check GitHub Action results from `.github/workflows/benchmark.yml`

Legacy baseline snapshot:

| impl | scenario | build_us | query_us | bits_per_key |
|---|---|---:|---:|---:|
| ribbon-filter | n=10000;w=16;r=8 | 2482 | 58281 | 32.0000 |
| fastbloom-rs | n=10000;w=16;r=8 | 260 | 20835 | 9.5872 |
| bloomfilter | n=10000;w=16;r=8 | 592 | 34790 | 9.5856 |
| bloomz | n=10000;w=16;r=8 | 555 | 58637 | 9.5851 |
| clubcard | n=10000;w=16;r=8 | 80951 | 73274 | 8.9408 |
| ribbon-filter | n=100000;w=16;r=8 | 33213 | 73376 | 32.0000 |
| fastbloom-rs | n=100000;w=16;r=8 | 2871 | 22333 | 9.5853 |
| bloomfilter | n=100000;w=16;r=8 | 4126 | 36718 | 9.5851 |
| bloomz | n=100000;w=16;r=8 | 5467 | 57789 | 9.5851 |
| clubcard | n=100000;w=16;r=8 | 194365 | 78431 | 5.3677 |
| ribbon-filter | n=100000;w=96;r=10 | 46893 | 173536 | 40.0000 |
| fastbloom-rs | n=100000;w=96;r=10 | 2434 | 22134 | 9.5853 |
| bloomfilter | n=100000;w=96;r=10 | 4179 | 36450 | 9.5851 |
| bloomz | n=100000;w=96;r=10 | 5253 | 57158 | 9.5851 |
| clubcard | n=100000;w=96;r=10 | 184790 | 77023 | 5.3779 |

Legacy notes:

- `build_us` is one build run for each implementation/scenario pair.
- `query_us` is one million negative probes.
- `bits_per_key` for `ribbon-filter` is computed as `(m * r) / n` from filter params.
- `bits_per_key` for `fastbloom-rs` and `bloomfilter` comes from their exposed total bit lengths.
- `bits_per_key` for `bloomz` is read from serialized metadata because the crate does not expose `m` directly.
- `clubcard` is benchmarked using the `clubcard` crate with the `builder` feature and a single synthetic partition.
- `clubcard` build includes both approximate and exact ribbon construction for positives and benchmark query negatives.
- `bits_per_key` for `clubcard` is estimated from `ApproximateSizeOf` heap footprint divided by inserted keys.
