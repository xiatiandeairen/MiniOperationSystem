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

### Non-obvious notes

- `volatile` v0.6 exports `VolatilePtr` and `VolatileRef`, **not** the old
  `Volatile<T>` wrapper from v0.4. For VGA/MMIO buffers, use `VolatilePtr::new(NonNull<T>)`.
- Crates using `extern "x86-interrupt"` handlers must enable
  `#![feature(abi_x86_interrupt)]` at their crate root.
- `pic8259` is not in the workspace dependencies. The HAL crate has a local
  `pic.rs` module that implements a minimal 8259 PIC driver using `x86_64`
  port I/O.
- `build-std` flags are **not** in `.cargo/config.toml`; they must be passed
  on the command line (the `Makefile.toml` tasks handle this automatically via
  `cargo make build` / `cargo make clippy`).

### Development workflow

Follow `plan.md` § 1.1–1.8 for the pre-check → develop → commit → review loop.
Commit messages use Conventional Commits: `<type>(<scope>): <what problem is solved>`.
