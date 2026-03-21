# Release Candidate Process

This project uses a lightweight `0.1.0-rc` to `0.1.0` path.

## Steps

1. Ensure CI is green on stable + MSRV.
2. Run local quality gates:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items`
   - `cargo test`
3. Run benchmark command and compare against baseline:
   - `cargo run --release --bin ribbon-bench`
4. Confirm no unresolved public API TODO markers in docs.
5. Publish `0.1.0-rc.1` and collect feedback.
6. If no blockers, update changelog and publish `0.1.0`.

## RC Exit Criteria

- No known correctness bugs for supported modes/widths.
- No flaky tests in CI suite.
- API docs and README are internally consistent.
- Performance is within expected envelope relative to baseline.
