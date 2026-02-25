# Changelog

## [CP1] — Project Scaffold & Build System

### Added
- Cargo workspace with 13 crates (common, hal, trace, trace-macros, memory,
  interrupt, process, scheduler, fs, ipc, syscall, shell, kernel).
- `rust-toolchain.toml` pinning nightly with required components.
- `.cargo/config.toml` targeting `x86_64-unknown-none` with `build-std`.
- `Makefile.toml` with build / test / clippy / fmt / run / debug tasks.
- `minios-common` crate: shared ID types, error enums, data types, and
  trait contracts for all subsystems.
- Documentation archive structure (`docs/adr/`, `docs/progress/`, etc.).
- ADR-001 (Rust nightly), ADR-002 (bootloader_api), ADR-003 (multi-crate).
- `.gitignore` excluding build artifacts.
