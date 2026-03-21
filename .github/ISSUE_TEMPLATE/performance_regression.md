---
name: Performance regression
about: Report throughput/latency/memory regression
title: "[perf] "
labels: performance
assignees: ''
---

## Regression summary

What regressed (build throughput, query throughput, bits/key, etc.).

## Measurement method

Command(s) used:

```sh
cargo run --release --bin ribbon-bench
```

Include machine details and rust toolchain.

## Before / After

- before:
- after:
- delta (%):

## Scenario details

- dataset shape:
- params (`m`, `w`, `r`, mode):
- seed/retry policy:

## Notes

Any links to commits, PRs, or profiling output.
