# Fuzz Targets

This directory contains cargo-fuzz harnesses for panic-resilience testing.

## Target

- `build_query`: exercises parameter parsing, builder construction, and membership queries across edge-sized parameter combinations.

## Run

```sh
cargo fuzz run build_query -- -max_total_time=60
```

## Notes

- The harness intentionally accepts both successful and failed constructions.
- A successful construction always performs several `contains_in` calls.
- Starter corpus is in `fuzz/corpus/build_query`.
