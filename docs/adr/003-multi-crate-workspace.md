# ADR-003: Multi-Crate Cargo Workspace Architecture

## Status
Accepted

## Background
The OS kernel has multiple subsystems (HAL, memory, process, fs, trace, …).
Code organisation affects testability, compile times, and module isolation.

## Decision
Use a Cargo workspace with one crate per subsystem. All cross-crate contracts
live in `minios-common` as trait definitions. Concrete implementations reside
in their respective crates and depend only on `minios-common` traits.

## Alternatives
1. **Single crate with modules** — simpler Cargo setup but weaker isolation;
   any module can access any other module's internals.
2. **Separate repositories** — too much overhead for a single-product project.

## Consequences
- Strong compile-time enforcement of module boundaries.
- Each crate can be unit-tested independently with mock trait implementations.
- Slightly more Cargo.toml boilerplate.
