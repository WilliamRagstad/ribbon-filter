# Benchmark Baseline

Generated with:

```sh
cargo run --release --bin ribbon-bench
```

Environment:

- date: 2026-03-21
- profile: release
- platform: windows x86_64 (msvc)

Results:

| scenario | build_us | query_us | bits_per_key |
|---|---:|---:|---:|
| n=10000;w=16;r=8 | 6616 | 58817 | 32.0000 |
| n=100000;w=16;r=8 | 73257 | 168896 | 32.0000 |
| n=100000;w=96;r=10 | 284203 | 221574 | 40.0000 |

Notes:

- `build_us` is one build run for that scenario.
- `query_us` is one million negative probes using `contains_in` and reusable scratch.
- `bits_per_key` is computed as `(m * r) / n` from filter params.
