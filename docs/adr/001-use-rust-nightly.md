# ADR-001: Use Rust Nightly Toolchain

## Status
Accepted

## Background
Bare-metal OS development in Rust requires unstable features such as
`#![no_std]`, `#![no_main]`, custom allocators, inline assembly, and
`build-std` for cross-compiling `core` and `alloc` to a freestanding target.

## Decision
Use `rustc` nightly channel, pinned via `rust-toolchain.toml`, with the
following components: `rust-src`, `llvm-tools-preview`, `rustfmt`, `clippy`.

## Alternatives
1. **Stable Rust** — cannot use required `#![no_main]`, `build-std`, naked
   functions, or custom allocators. Not viable for OS development.
2. **Specific nightly date pin** — more reproducible but harder to maintain;
   we rely on `rust-toolchain.toml` for reproducibility instead.

## Consequences
- CI must install `nightly`.
- Occasional breakage from nightly changes is possible; mitigate by updating
  intentionally at checkpoint boundaries.
