# MiniOS — Learn Operating Systems by Doing

> An interactive teaching OS. Boot it, type commands, see how an OS works from the inside.

## 30-Second Start

```bash
cargo make run-gui    # Boot with graphical window
```

You'll see a shell prompt. Try these:

```
MiniOS $ tutorial           ← guided 10-step learning path
MiniOS $ explain ls         ← learn how a command works internally
MiniOS $ trace follow ls /  ← see every system call in a command
MiniOS $ compare scheduler  ← MiniOS vs Linux design differences
MiniOS $ lab memory-usage   ← hands-on memory experiment
MiniOS $ crash oom          ← safely trigger out-of-memory
MiniOS $ journey            ← track your learning progress
```

## What You'll Learn

| Topic | Commands | Concept |
|-------|----------|---------|
| Processes | ps, spawn, kill, sched | PCB, scheduling, context switch |
| Memory | meminfo, frames, pagetable, alloc | Pages, frames, heap allocation |
| Filesystem | ls, cat, mkdir, write | VFS, inodes, file descriptors |
| Tracing | trace follow, trace tree, log | Spans, observability, debugging |
| Faults | crash oom, crash stack | OOM, page fault, stack overflow |
| Design | compare, explain | MLFQ vs CFS, RamFS vs ext4 |

## 44 Shell Commands

Type `help` to see all commands. Type `explain <cmd>` to learn how any command works.

## Zero-Setup with Docker
```bash
docker build -t minios . && docker run -it --rm minios
```

## Prerequisites

- Rust nightly (`rust-toolchain.toml` handles this)
- QEMU (`apt install qemu-system-x86`)
- cargo-make (`cargo install cargo-make`)

## Project Structure

13-crate Rust workspace. See [project.md](project.md) for architecture details.

## Links

- [Tutorial](docs/releases/) — Version history
- [spec.md](spec.md) — Technical specification
- [claude.md](claude.md) — Development rules
