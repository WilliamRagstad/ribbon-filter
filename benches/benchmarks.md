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

| scenario | build_us | query_us | bits_per_key |
|---|---:|---:|---:|
| n=10000;w=16;r=8 | 5087 | 46002 | 32.0000 |
| n=100000;w=16;r=8 | 71630 | 93647 | 32.0000 |
| n=100000;w=96;r=10 | 242429 | 225178 | 40.0000 |

Notes:

- `build_us` is one build run for that scenario.
- `query_us` is one million negative probes using `contains_in` and reusable scratch.
- `bits_per_key` is computed as `(m * r) / n` from filter params.
