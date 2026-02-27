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
- **Stack overflow in debug builds**: The default bootloader kernel stack is
  80 KiB, which is too small for the memory subsystem's bitmap frame allocator
  (8 KiB `BitmapInner` struct) combined with x86_64 page table operations in
  debug mode. The kernel `BootloaderConfig` sets `kernel_stack_size = 512 KiB`.
  If you add subsystems with large stack frames, watch for double faults during
  boot — they almost always indicate stack overflow.
- **QEMU boot testing**: Use `cargo make run` for a one-command release
  build + boot, or build and boot manually:
  ```
  cargo make build-release
  ./tools/boot-image/target/x86_64-unknown-linux-gnu/release/boot-image target/x86_64-unknown-none/release/minios-kernel
  timeout 15 qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/release/minios-bios.img -nographic -m 256M -no-reboot -no-shutdown
  ```
  Debug builds with full debug info (~8 MB+) exceed the BIOS bootloader
  stage-2 limit and panic during kernel load. Always use **release builds**
  for QEMU testing.
- **Shell keyboard input in QEMU**: The shell reads from the PS/2 keyboard
  scancode port (0x60). In `-nographic` mode, terminal input goes to serial,
  not the PS/2 keyboard. The shell will start but appear to hang waiting for
  input. Use `-display gtk` or similar for interactive keyboard testing.

- **`minios-shell` host tests**: The shell crate links `minios-memory` which
  declares a `#[global_allocator]` using `LockedHeap::empty()`. On host
  (`x86_64-unknown-linux-gnu`) this allocator is never initialised, so the test
  binary SIGABRTs before any test runs. Pure-logic tests (parser, `contains_pattern`)
  compile fine but cannot execute. Only crates without a custom allocator
  (`minios-common`, `minios-trace`, `minios-scheduler`, `minios-ipc`) are
  testable on the host target.

- **klog! and heap init**: The `klog!` macro checks the log level before
  `format!()` to avoid heap allocation when the message would be discarded.
  Any code that runs before `minios_memory::init()` must not call heap-allocating
  functions. This includes `alloc::format!`, `alloc::vec!`, `Box::new`, etc.
- **Multitasking architecture (v2.0)**: The Shell runs as a real scheduled task
  (PID 1), not directly in `kernel_main`. Context switching is deferred: the
  timer ISR sets `NEED_RESCHEDULE` / `NEXT_TASK_PID` flags, and the actual
  switch happens at `hlt()` return points — safe yield points where no Mutex
  is held. Never call `switch_context` inside an ISR.
- **Host-side tests**: The `cargo make test` task runs in workspace mode which
  fails. Use the direct command instead:
  `cargo test -p minios-common -p minios-trace-macros -p minios-scheduler -p minios-ipc --target x86_64-unknown-linux-gnu`

### Development workflow

Follow `plan.md` § 1.1–1.8 for the pre-check → develop → commit → review loop.
Commit messages use Conventional Commits: `<type>(<scope>): <what problem is solved>`.
