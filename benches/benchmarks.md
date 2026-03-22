# Benchmark Baseline

Generated with:

```sh
cargo bench --bench ribbon
```

Environment:

- date: 2026-03-21
- profile: release
- platform: windows x86_64 (msvc)

Results:

| impl | scenario | build_us | query_us | bits_per_key |
|---|---|---:|---:|---:|
| ribbon-filter | n=10000;w=16;r=8 | 2180 | 58688 | 32.0000 |
| fastbloom-rs | n=10000;w=16;r=8 | 290 | 21093 | 9.5872 |
| bloomfilter | n=10000;w=16;r=8 | 727 | 35712 | 9.5856 |
| bloomz | n=10000;w=16;r=8 | 556 | 52397 | 9.5851 |
| ribbon-filter | n=100000;w=16;r=8 | 31957 | 79722 | 32.0000 |
| fastbloom-rs | n=100000;w=16;r=8 | 2704 | 22792 | 9.5853 |
| bloomfilter | n=100000;w=16;r=8 | 4766 | 37575 | 9.5851 |
| bloomz | n=100000;w=16;r=8 | 5162 | 53958 | 9.5851 |
| ribbon-filter | n=100000;w=96;r=10 | 48619 | 210208 | 40.0000 |
| fastbloom-rs | n=100000;w=96;r=10 | 2836 | 23257 | 9.5853 |
| bloomfilter | n=100000;w=96;r=10 | 5429 | 41015 | 9.5851 |
| bloomz | n=100000;w=96;r=10 | 5542 | 54524 | 9.5851 |

Notes:

- `build_us` is one build run for each implementation/scenario pair.
- `query_us` is one million negative probes.
- `bits_per_key` for `ribbon-filter` is computed as `(m * r) / n` from filter params.
- `bits_per_key` for `fastbloom-rs` and `bloomfilter` comes from their exposed total bit lengths.
- `bits_per_key` for `bloomz` is read from serialized metadata because the crate does not expose `m` directly.
