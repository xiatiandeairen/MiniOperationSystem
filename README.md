# MiniOperationSystem

A micro operating system kernel for x86_64, written in Rust.

**Core features:** decoupled trait-based architecture · full-chain trace
logging · browser-based trace visualization.

## Prerequisites

- Rust nightly (auto-managed by `rust-toolchain.toml`)
- [QEMU](https://www.qemu.org/) (`qemu-system-x86_64`)
- [cargo-make](https://github.com/sagiegurari/cargo-make)
  (`cargo install cargo-make`)

## Quick Start

```bash
cargo make build        # compile all crates
cargo make test         # run unit tests
cargo make clippy       # lint
cargo make fmt          # format code
cargo make run          # boot in QEMU (headless)
cargo make run-display  # boot in QEMU with VGA window
```

## Project Structure

```
crates/
├── common/       # shared types, traits, errors
├── hal/          # hardware abstraction layer
├── trace/        # trace engine (ring buffer, spans)
├── trace-macros/ # #[traced] proc-macro
├── memory/       # physical + virtual memory management
├── interrupt/    # IDT, PIC, exception handlers
├── process/      # PCB, context switch
├── scheduler/    # MLFQ scheduler
├── fs/           # VFS + RamFS + ProcFS + TraceFS
├── ipc/          # message queues, shared memory
├── syscall/      # system call dispatcher
├── shell/        # interactive terminal
└── kernel/       # boot entry point, integration
```

See [spec.md](spec.md) for the full technical specification and
[plan.md](plan.md) for the development plan.
