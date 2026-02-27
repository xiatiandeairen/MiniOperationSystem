//! System information commands: uptime, meminfo, sleep, version, safety,
//! snapshot, interrupts, and stats.

use core::sync::atomic::{AtomicU64, Ordering};
use minios_hal::println;

static SNAPSHOT_FRAMES: AtomicU64 = AtomicU64::new(0);
static SNAPSHOT_HEAP: AtomicU64 = AtomicU64::new(0);
static SNAPSHOT_TICK: AtomicU64 = AtomicU64::new(0);

/// Shows uptime in human-readable minutes and seconds.
pub fn cmd_uptime(_args: &[&str]) {
    let ticks = minios_hal::interrupts::tick_count();
    let seconds = ticks / 100;
    let minutes = seconds / 60;
    println!("Uptime: {}m {}s ({} ticks)", minutes, seconds % 60, ticks);
    super::journey::mark(super::journey::STEP_UPTIME);
}

/// Displays memory statistics (frame allocator + heap).
pub fn cmd_meminfo(_args: &[&str]) {
    let stats = minios_memory::get_stats();
    println!(
        "Frames: {} free / {} total ({} KiB free)",
        stats.free_frames,
        stats.total_frames,
        stats.free_frames * 4,
    );
    println!(
        "Heap:   {} used / {} free",
        stats.heap_used, stats.heap_free
    );
    super::journey::mark(super::journey::STEP_MEMINFO);
}

/// Sleeps for the specified number of ticks (default 100).
pub fn cmd_sleep(args: &[&str]) {
    let ticks = if args.is_empty() {
        100
    } else {
        args[0]
            .bytes()
            .fold(0u64, |a, b| {
                if b.is_ascii_digit() {
                    a * 10 + (b - b'0') as u64
                } else {
                    a
                }
            })
            .max(1)
    };
    let start = minios_hal::interrupts::tick_count();
    println!("Sleeping for {} ticks...", ticks);
    while minios_hal::interrupts::tick_count() - start < ticks {
        minios_hal::cpu::hlt();
    }
    println!(
        "Awake! (slept {} ticks)",
        minios_hal::interrupts::tick_count() - start
    );
}

/// Displays MiniOS version and system information.
pub fn cmd_version(_args: &[&str]) {
    println!("MiniOS v1.0");
    println!("Architecture: x86-64 (bare metal)");
    println!("Shell commands: {}", super::list_commands().len());
    println!("Subsystems: HAL, Trace, Memory, Process, Scheduler, FS, IPC, Syscall, Shell");
    println!("Tests: 105+");
    println!("Build: Rust nightly, bootloader_api 0.11");
}

/// Prints an audit summary of unsafe code usage in MiniOS.
pub fn cmd_safety(_args: &[&str]) {
    println!("=== MiniOS Safety Audit ===");
    println!();
    println!("Unsafe code locations:");
    println!("  hal/gdt.rs      \u{2014} GDT/TSS static stack (SAFETY: one-time init)");
    println!("  hal/vga.rs      \u{2014} VGA buffer pointer (SAFETY: hardware-mapped)");
    println!("  hal/framebuffer \u{2014} raw pointer to bootloader framebuffer");
    println!("  hal/serial.rs   \u{2014} UART port I/O (SAFETY: standard COM1 address)");
    println!("  hal/cpu.rs      \u{2014} inline asm for TSC/HLT (SAFETY: privileged ops)");
    println!("  process/context \u{2014} switch_context_asm (SAFETY: callee-saved regs)");
    println!("  memory/frame    \u{2014} bitmap from bootloader memory map");
    println!("  memory/paging   \u{2014} page table from CR3 register");
    println!("  memory/heap     \u{2014} heap init from raw pointer");
    println!();
    println!("Safety invariants:");
    println!("  - All Mutex-protected data is Send+Sync");
    println!("  - No unsafe in shell/fs/ipc/scheduler/syscall crates");
    println!("  - Double-free protected in frame deallocator");
    println!("  - ISR never acquires Mutex (deadlock prevention)");
}

/// Saves or diffs a system state snapshot for comparison over time.
pub fn cmd_snapshot(args: &[&str]) {
    if args.is_empty() || args[0] == "save" {
        let stats = minios_memory::get_stats();
        let tick = minios_hal::interrupts::tick_count();
        SNAPSHOT_FRAMES.store(stats.free_frames as u64, Ordering::Relaxed);
        SNAPSHOT_HEAP.store(stats.heap_used as u64, Ordering::Relaxed);
        SNAPSHOT_TICK.store(tick, Ordering::Relaxed);
        println!("Snapshot saved at tick {}", tick);
    } else if args[0] == "diff" {
        let old_frames = SNAPSHOT_FRAMES.load(Ordering::Relaxed);
        let old_heap = SNAPSHOT_HEAP.load(Ordering::Relaxed);
        let old_tick = SNAPSHOT_TICK.load(Ordering::Relaxed);
        if old_tick == 0 {
            println!("No snapshot saved. Use 'snapshot save' first.");
            return;
        }
        let stats = minios_memory::get_stats();
        let tick = minios_hal::interrupts::tick_count();
        println!("=== State Diff (tick {} \u{2192} {}) ===", old_tick, tick);
        let frame_diff = stats.free_frames as i64 - old_frames as i64;
        let heap_diff = stats.heap_used as i64 - old_heap as i64;
        println!(
            "  Frames free: {} \u{2192} {} ({:+})",
            old_frames, stats.free_frames, frame_diff
        );
        println!(
            "  Heap used:   {} \u{2192} {} ({:+})",
            old_heap, stats.heap_used, heap_diff
        );
        println!("  Ticks elapsed: {}", tick - old_tick);
    } else {
        println!("Usage: snapshot [save|diff]");
    }
}

/// Shows interrupt statistics (timer and keyboard counters).
pub fn cmd_interrupts(_args: &[&str]) {
    let stats = minios_hal::interrupts::interrupt_stats();
    let uptime_secs = stats.timer_count / 100;
    println!("IRQ  NAME       COUNT     RATE");
    println!("0    Timer      {:<9} ~100/s", stats.timer_count);
    println!("1    Keyboard   {:<9} on-demand", stats.keyboard_count);
    println!();
    println!(
        "Uptime: ~{} seconds ({} ticks)",
        uptime_secs, stats.timer_count
    );
}

/// Displays session statistics: command count, journey progress, uptime.
pub fn cmd_stats(_args: &[&str]) {
    let cmds = crate::shell::COMMAND_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    let journey = crate::commands::journey::completed_count();
    let uptime = minios_hal::interrupts::tick_count();
    println!("=== Session Statistics ===");
    println!("  Commands executed: {}", cmds);
    println!("  Journey progress:  {}/17", journey);
    println!("  Session uptime:    {} ticks (~{}s)", uptime, uptime / 100);
}
