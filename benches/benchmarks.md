# Benchmark Baseline

Generated with:

```sh
cargo bench --bench ribbon
```

Environment:

- date: 2026-03-22
- profile: release
- platform: windows x86_64 (msvc)

Results:

| impl | scenario | build_us | query_us | bits_per_key |
|---|---|---:|---:|---:|
| ribbon-filter | n=10000;w=16;r=8 | 2180 | 58688 | 32.0000 |
| fastbloom-rs | n=10000;w=16;r=8 | 290 | 21093 | 9.5872 |
| bloomfilter | n=10000;w=16;r=8 | 727 | 35712 | 9.5856 |
| bloomz | n=10000;w=16;r=8 | 556 | 52397 | 9.5851 |
| clubcard | n=10000;w=16;r=8 | 76605 | 73912 | 8.9472 |
| ribbon-filter | n=100000;w=16;r=8 | 33412 | 75290 | 32.0000 |
| fastbloom-rs | n=100000;w=16;r=8 | 2573 | 22824 | 9.5853 |
| bloomfilter | n=100000;w=16;r=8 | 4807 | 37775 | 9.5851 |
| bloomz | n=100000;w=16;r=8 | 4395 | 44264 | 9.5851 |
| clubcard | n=100000;w=16;r=8 | 191544 | 78940 | 5.3715 |
| ribbon-filter | n=100000;w=96;r=10 | 47503 | 178001 | 40.0000 |
| fastbloom-rs | n=100000;w=96;r=10 | 2634 | 23140 | 9.5853 |
| bloomfilter | n=100000;w=96;r=10 | 4761 | 38271 | 9.5851 |
| bloomz | n=100000;w=96;r=10 | 4230 | 45677 | 9.5851 |
| clubcard | n=100000;w=96;r=10 | 212341 | 78482 | 5.3741 |

Notes:

- `build_us` is one build run for each implementation/scenario pair.
- `query_us` is one million negative probes.
- `bits_per_key` for `ribbon-filter` is computed as `(m * r) / n` from filter params.
- `bits_per_key` for `fastbloom-rs` and `bloomfilter` comes from their exposed total bit lengths.
- `bits_per_key` for `bloomz` is read from serialized metadata because the crate does not expose `m` directly.
- `clubcard` is benchmarked using the `clubcard` crate with the `builder` feature and a single synthetic partition.
- `clubcard` build includes both approximate and exact ribbon construction for positives and benchmark query negatives.
- `bits_per_key` for `clubcard` is estimated from `ApproximateSizeOf` heap footprint divided by inserted keys.
