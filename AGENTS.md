# AGENTS.md

## Cursor Cloud specific instructions

This is a Rust bare-metal OS kernel targeting `x86_64-unknown-none`.

### Build & test commands

All tasks are defined in `Makefile.toml`. Run via `cargo make <task>`:

| Command | Purpose |
|---------|---------|
| `cargo make build` | Compile workspace |
| `cargo make test` | Host-side unit tests |
| `cargo make clippy` | Lint (warnings = errors) |
| `cargo make fmt-check` | Check formatting |
| `cargo make ci` | fmt-check + clippy + test |
| `cargo make run` | Boot in QEMU (headless, serial on stdout) |

### Key caveats

- The default Cargo target is `x86_64-unknown-none` (set in `.cargo/config.toml`).
  `build-std` is enabled globally, so `core` and `alloc` are cross-compiled automatically.
- `crates/trace-macros` is a proc-macro crate and is always compiled for the
  host. It does not use `#![no_std]`.
- The `bootloader` crate (which requires `std`) is **not** a cargo dependency.
  Only `bootloader_api` (no_std-compatible) is used by the kernel. Disk image
  creation is a separate build step.
- When adding a new subsystem, define its trait in `crates/common/src/traits/`
  and implement it in its own crate. Never depend on a concrete implementation
  from another crate.

### Development workflow

Follow `plan.md` § 1.1–1.8 for the pre-check → develop → commit → review loop.
Commit messages use Conventional Commits: `<type>(<scope>): <what problem is solved>`.
