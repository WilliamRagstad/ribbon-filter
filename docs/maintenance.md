# Post-release Maintenance Checklist

## Routine checks

- Keep CI green on stable and MSRV.
- Review failed fuzz runs and triage crashes.
- Run benchmark baseline command periodically:
  - `cargo run --release --bin ribbon-bench`

## Performance regression policy

When a regression is reported:

1. Reproduce with `ribbon-bench` and same params/seed.
2. Compare against `regression/benchmarks.md` and prior commit output.
3. If query/build regression exceeds 10% in repeated runs, open a perf issue.
4. Land either a fix or an explicit acceptance note in changelog.

## Issue triage

- Use `bug` template for correctness/API behavior defects.
- Use `performance` template for throughput/latency/memory regressions.
- Add minimal reproducer test where possible.

## Release hygiene

- Update changelog for every user-visible change.
- Keep README guarantees/limits in sync with implementation.
- Avoid merging public API changes without docs/tests updates.
