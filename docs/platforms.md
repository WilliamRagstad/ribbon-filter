# Supported Toolchain and Platform Matrix

## Rust toolchains

- **MSRV**: Rust `1.86.0`
- **Stable**: latest stable Rust

CI enforces both:

- `msrv` job runs `cargo check` on `1.86.0`
- `quality` job runs formatting, clippy, docs, and tests on stable

## Target matrix policy

Primary support targets for this crate:

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `x86_64-apple-darwin`

Current CI executes on Linux only; other targets are validated on-demand and tracked as release readiness criteria.

## Notes

- The crate is `unsafe`-free (`#![forbid(unsafe_code)]`).
- Fuzzing requires nightly + `cargo-fuzz` and platform sanitizer toolchain support.
