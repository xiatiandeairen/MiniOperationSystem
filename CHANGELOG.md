# Changelog

## [CP10] — Trace Visualization & Final Integration

### Added
- `trace-viewer/`: standalone HTML/CSS/JS web application for visualizing
  trace data exported from MiniOS (waterfall chart, stats panel, drag-and-drop
  file loading, dark developer theme).
- `scripts/integration-test.sh`: end-to-end build-and-boot smoke test that
  verifies kernel subsystem initialization messages.
- `docs/progress/cp10-report.md`: final project completion report.
- CHANGELOG entries for all 10 checkpoints.

## [CP9] — Shell Interactive Terminal

### Added
- `crates/shell`: command parser, line editor with backspace/Ctrl-C,
  prompt showing PID and cwd.
- Built-in commands: help, echo, clear, ls, cd, pwd, cat, mkdir, touch,
  rm, write, ps, meminfo, uptime, kill.
- Trace commands: `trace list`, `trace tree`, `trace stats`, `trace filter`,
  `trace export`, `trace clear`, `trace live`.
- Shell integrated as init process (PID 1) during boot.

## [CP8] — System Calls & IPC

### Added
- `crates/syscall`: system call dispatcher registered via IDT software
  interrupt, parameter passing through registers, return value convention.
- I/O syscalls: read, write, open, close, stat.
- Process syscalls: fork, exit, getpid, yield, waitpid.
- Memory syscalls: mmap, munmap, meminfo.
- Trace syscalls: trace_dump, trace_config.
- `crates/ipc`: message queue IPC (create, send, receive, destroy) and
  simplified shared-memory regions.
- IPC TraceContext propagation across process boundaries.

## [CP7] — File System

### Added
- `crates/fs`: VFS layer with path resolution, mount-point management,
  file descriptor table.
- RamFS: in-memory filesystem with inode-based storage, file create / read /
  write, directory operations.
- ProcFS: virtual `/proc` with meminfo, cpuinfo, uptime, process status.
- TraceFS: virtual `/trace` exposing current ring buffer, config, stats.
- DevFS: `/dev/null`, `/dev/zero`, `/dev/console`, `/dev/serial`.
- Initial directory tree: `/dev`, `/proc`, `/tmp`, `/trace`, `/etc`.

## [CP6] — Process Management & Scheduler

### Added
- `crates/process`: PCB with atomic PID allocation, kernel-stack allocation,
  CpuContext save/restore, context-switch assembly stub.
- ProcessManager: process table (BTreeMap), create / exit / kill / list.
- `crates/scheduler`: Multi-Level Feedback Queue (MLFQ) with 4 priority
  queues, time-slice management, priority demotion, periodic boost.
- Idle task (PID 0) running `hlt` loop.
- Timer-interrupt–driven preemptive scheduling.
- Init process (PID 1) creation during boot.

## [CP5] — Interrupts & Exception Handling

### Added
- `crates/interrupt`: IDT setup with handlers for Division Error, Invalid
  Opcode, Double Fault (IST), General Protection Fault, Page Fault.
- Timer interrupt (IRQ 0) with system tick counter.
- PS/2 keyboard interrupt (IRQ 1) with scancode → ASCII conversion,
  Shift/Ctrl/CapsLock modifiers, event queue.
- InterruptManager trait implementation, dynamic handler registration,
  per-IRQ statistics.
- Interrupts enabled (`sti`) at end of boot sequence.

## [CP4] — Memory Management

### Added
- `crates/memory`: bitmap-based physical frame allocator initialized from
  BootInfo memory map.
- 4-level page table virtual memory manager: translate, map, unmap,
  create_address_space.
- Linked-list kernel heap allocator implementing `GlobalAlloc`.
- MemoryManager façade unifying frame allocator, VMM, and heap.
- Trace instrumentation on every alloc / dealloc / map / unmap operation.
- Boot-time memory statistics printed to VGA and serial.

## [CP3] — Trace Engine

### Added
- `crates/trace`: Span data structure, SpanAttributes (zero-allocation),
  TraceContext, SpanId/TraceId generators, ring buffer (64 K slots),
  SpanFilter, SpanGuard RAII, TraceConfig, TraceStats, NullTracer.
- `trace_span!` and `trace_event!` macros.
- JSON serialization for spans and trace export.
- Serial trace output channel.
- TSC frequency calibration.
- `crates/trace-macros`: `#[traced]` proc-macro attribute with module
  parameter and return-value capture.

## [CP2] — HAL & Boot

### Added
- `crates/hal`: Port I/O wrappers, serial driver (COM1 init / read / write),
  VGA text-mode driver (character write, scroll, color, cursor, clear,
  fmt::Write), GDT + TSS, PIC 8259 driver, CPU control functions
  (hlt, interrupts_enabled, without_interrupts).
- `crates/kernel`: bootloader_api entry point (`_start`), panic handler
  with serial + VGA output, QEMU debug-exit, custom test framework.
- HalSerial, HalDisplay, HalInterruptController trait implementations.
- BIOS boot image generation via `boot-image` tool.

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
