# MiniOperationSystem

A micro operating system kernel for x86_64, written in Rust.

**Features:** decoupled trait-based architecture · full-chain trace
logging · interactive shell · browser-based trace visualization.

## Quick Start

```bash
# Prerequisites: Rust nightly + QEMU
cargo make run          # headless (serial on stdout)
cargo make run-gui      # graphical QEMU window with shell
```

First run will auto-build the boot image tool (~60s one-time cost).

## What You'll See

On boot, MiniOS initialises all subsystems and drops into a shell:

```
Hello, MiniOS!
Boot successful. System ready.
MiniOS Shell v0.1
Type 'help' for available commands.

MiniOS $ help
Available commands:
  help     - List all available commands
  echo     - Print arguments to the screen
  clear    - Clear the VGA screen
  uptime   - Show tick count since boot
  meminfo  - Show memory statistics
  ls       - List directory contents
  cat      - Print file contents
  mkdir    - Create a directory
  touch    - Create an empty file
  write    - Write content to a file
  pwd      - Print working directory
  ps       - List all processes
  trace    - Trace subsystem (list|tree|stats|clear|export)
```

## Architecture

13-crate Cargo workspace — each subsystem isolated behind a trait interface:

```
crates/
├── common/       # shared types, traits, errors
├── hal/          # serial, framebuffer console, GDT, IDT, PIC, keyboard
├── trace/        # ring buffer trace engine, JSON export
├── memory/       # bitmap frame allocator, page tables, heap
├── interrupt/    # (integrated into HAL)
├── process/      # PCB, context switch assembly
├── scheduler/    # MLFQ 4-level scheduler
├── fs/           # VFS + RamFS + ProcFS
├── ipc/          # message queues with TraceContext propagation
├── syscall/      # syscall dispatcher
├── shell/        # 15-command interactive terminal
└── kernel/       # boot entry, subsystem wiring
```

## Development Commands

| Command | Purpose |
|---------|---------|
| `cargo make build` | Compile (bare-metal cross-compile) |
| `cargo make build-release` | Release build |
| `cargo make test` | Host-side unit tests (59 tests) |
| `cargo make clippy` | Lint (warnings = errors) |
| `cargo make fmt` | Format code |
| `cargo make ci` | fmt + clippy + test |
| `cargo make run` | Build + boot in QEMU (headless) |
| `cargo make run-gui` | Build + boot with graphical window |
| `cargo make run-trace` | Boot + capture trace to file |
| `cargo make debug` | Boot + wait for GDB on port 1234 |

## Trace Viewer

Open `trace-viewer/index.html` in a browser to visualise trace data.
Use `trace export` in the MiniOS shell to dump spans as JSON via serial.

## Documentation

- [spec.md](spec.md) — full technical specification
- [plan.md](plan.md) — development plan (500 tasks, 10 checkpoints)
- [claude.md](claude.md) — AI self-driving development rules
- [project.md](project.md) — project status and index
- [docs/tasks/](docs/tasks/) — iteration plans and results
