# Changelog

## Rounds 16–18 — Shell polish + boot banner
- Live memory stats in `meminfo` command
- `clear` command works with framebuffer
- Colorful ASCII art boot banner
- Updated README quickstart, project.md, CHANGELOG

## Rounds 13–15 — Color output + trace tree + README
- Framebuffer color API with predefined colors (green/red/yellow/white)
- Shell prompt in green, errors in red
- `trace tree` command: indented span hierarchy with cycle counts
- README rewritten as developer quickstart guide

## Rounds 10–12 — Trace completeness + ls sizes
- IPC Message carries TraceContext for cross-process trace linking
- Shell `trace export` serializes ring buffer as JSON to serial port
- `ls` shows file type (d/-/c/s) and size in bytes

## Rounds 7–9 — Test coverage expansion
- 8 MLFQ scheduler tests (tick, boost, preempt, stats)
- 8 RamFS tests (CRUD, lookup, duplicate detection)
- 5 IPC MessageQueue tests (FIFO, capacity, truncation)

## Rounds 4–6 — Code correctness
- Fixed misleading "lock-free" comment on mutex-protected ring buffer
- ProcFS generates live memory/uptime data (not cached placeholders)
- Scheduler boost_all reduced from O(n²) to O(n)

## ITER-003 — Interactive shell fix
- Fixed context switch preempting kernel_main → shell now responds to keyboard
- Cooperative scheduling: Shell runs as main loop, tasks tracked but not preempted

## ITER-002 — One-command run + framebuffer console
- `cargo make run` / `cargo make run-gui` one-command build+boot
- 8×16 bitmap font framebuffer text renderer (160×45 chars, dark theme)
- Boot messages and shell prompt visible in QEMU graphical window

## CP1–CP10 — Initial implementation
- 13-crate workspace with trait-based decoupled architecture
- HAL: serial, VGA, GDT/TSS, IDT (6 handlers), PIC 8259, keyboard
- Trace engine: 4096-slot ring buffer, SpanGuard RAII, JSON export
- Memory: bitmap frame allocator, 4-level page tables, 1 MiB heap
- Process: PCB, context switch (global_asm), PID allocator
- Scheduler: MLFQ 4-level with time slices and priority boost
- Filesystem: VFS + RamFS + ProcFS + file descriptor table
- Syscall: dispatcher with 7 system calls
- IPC: message queue with send/receive
- Shell: 15 commands including trace viewer integration
- Trace Viewer: browser-based waterfall chart (HTML/CSS/JS)
