<div align="center">
  <h1>🎗️ Ribbon Filter</h1>
  <h4>
    A static, space-efficient Ribbon filter implementation for Rust
    <br>
    <a href="#usage">Usage</a>
    <span> | </span>
    <a href="#current-feature-status">Features</a>
    <span> | </span>
    <a href="#testing">Quality</a>
  </h4>
</div>

<!-- 
- **Static membership filter**: build once from a key-set, then query fast.
- **Configurable false-positive behavior**: controlled by fingerprint width `r`.
- **Deterministic for fixed inputs**: same params + key-set + hasher => stable results.
- **Robust construction path**: reseed retries and optional `m` growth policy.
- **Bit-packed matrix representation**: compact storage using `bitvec`.
- **Mode support**: `Standard` and `Homogeneous`.
- **Width classes**: supports both `w <= 64` and `65..=128`.
-->

> [!WARNING]
> This crate is under active development and is **not suitable for production use yet**.
> APIs, behavior details, and internals may change with emphasis on correctness and hardening.

## Summary

This crate implements a **static approximate-membership filter** based on Ribbon-style construction over $GF(2)$ / $\mathbb{F}_2$ using bit-packed matrices. It supports two modes of operation *(standard and homogeneous)*, configurable parameters for filter size, fingerprint bit-width, retry/growth policies, and more.
The crate is designed around practical engineering constraints:

- deterministic construction and query behavior for fixed inputs,
- no false negatives after a successful build,
- compact memory representation for stored filter state,
- robust construction with retry and growth policy controls.

This crate targets workloads where keys are known up front and then queried many times.

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
ribbon-filter = "0.1"
```

Then build and query a filter:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
use ribbon_filter::{Mode, Params, RibbonBuilder};
type H = BuildHasherDefault<DefaultHasher>;
type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
  let params = Params::new(3000, 16, 10, Mode::Standard)?
      .with_seed(42)
      .with_retry_policy(4, 1)?;
  let builder = RibbonBuilder::new(params, H::default())?;
  // Our key-set (e.g. ints from 0 to 999)
  let keys: Vec<u64> = (0..1000).collect();
  let filter = builder.build(&keys)?;

  assert!(filter.contains(&123u64));
  Ok(())
}
```
<!-- 
  // allocation-free query scratch space
  let mut scratch = filter.new_scratch();
  assert!(filter.contains_in(&123u64, &mut scratch));
-->

## What This Crate Is

This crate is a Rust implementation of a Ribbon filter builder/query pipeline with:

- `Mode::Standard` (fingerprint equality checks),
- `Mode::Homogeneous` (zero right-hand-side constraints),
- widths `w` in `1..=128`,
- one-shot plus retry/grow construction behavior,
- bit-packed final filter matrix storage.

It is **not** a dynamic insert/delete structure and it is **not** a general-purpose probabilistic collection API yet.

| Guarantees | Limits |
|---|---|
| Build success implies **no false negatives** for inserted keys. | Static structure: no online insert/delete API. |
| Query behavior is deterministic for fixed hasher + params + key-set. | Current width support is capped at `128`. |
| Construction failures are surfaced with structured diagnostics. | Public API and internals are still evolving toward release-hardening milestones. |

## Parameter Guide

- `m`: number of rows in the solved matrix (`Z`); larger values generally ease construction.
- `w`: ribbon width (`1..=128` in current implementation).
- `r`: fingerprint bits; larger `r` generally means lower false positives.
- `seed`: base seed for deterministic attempt derivation.
- `retry_limit`: number of reseed attempts per growth level.
- `grow_limit`: number of `m` growth rounds (`ceil(m * (w + 1) / w)`).

Helper constructors:

- `Params::r_from_fpr(fpr)`: derive `r` for a target false-positive rate.
- `Params::from_expected_items(n, overhead, w, r, mode)`: derive `m` for expected item count and overhead target.

## Modes

- `Mode::Standard`: Uses generated fingerprints (`b(x)`) and checks query result equality, expected false-positive behavior follows the configured fingerprint width.

- `Mode::Homogeneous`: Uses zero right-hand-side constraints, construction path is simplified for this mode, retained keys still satisfy no-false-negative behavior in tests.

## Features

- [x] Standard mode construction and queries
- [x] Homogeneous mode construction and queries
- [x] Width support through `128`
- [x] Retry + growth construction policy
- [x] Allocation-free query API (`contains_in` + `Scratch`)
- [x] Bit-packed final storage
- [x] Compatibility test matrix (mode/width/r)
- [x] Statistical false-positive checks
- [x] Generated invariant/determinism tests
- [x] Fuzz harness and starter corpus
- [x] Adversarial regression corpus
- [x] CI gates for fmt/clippy/docs/tests

&nbsp;

## Testing

This repository includes: unit and integration-style behavior tests, statistical false-positive guardrails, generated invariant and determinism test cases, adversarial pattern regression coverage, CI checks for formatting, linting, docs, and tests.
Run all checks locally:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items
cargo test
```

## Benchmarks

Run reproducible baseline benchmarks:

```sh
cargo bench --bench ribbon
```

Committed baseline numbers are in `benches/benchmarks.md`.

## Fuzzing

Fuzz harnesses are under `fuzz/`.
Run the primary target:

```sh
rustup toolchain install nightly
cargo +nightly install cargo-fuzz
cargo +nightly fuzz list # target discovery
cargo +nightly fuzz run build_query -- -max_total_time=60
cargo +nightly fuzz run build_query_structured -- -max_total_time=60
```

The `build_query` harness stresses parameter decoding, construction paths, and query calls across edge-shaped inputs.
The `build_query_structured` harness uses `Arbitrary`-driven typed inputs to perform structure-aware fuzzing over params, keys, and query behavior.
