//! Basic shell commands: help, echo, clear, uptime, meminfo.

extern crate alloc;

use core::sync::atomic::{AtomicU64, Ordering};
use minios_hal::println;

static SNAPSHOT_FRAMES: AtomicU64 = AtomicU64::new(0);
static SNAPSHOT_HEAP: AtomicU64 = AtomicU64::new(0);
static SNAPSHOT_TICK: AtomicU64 = AtomicU64::new(0);

/// Lists all available commands with descriptions.
pub fn cmd_help(_args: &[&str]) {
    println!("Available commands:");
    for cmd in super::list_commands() {
        println!("  {:10} - {}", cmd.name, cmd.description);
    }
    super::journey::mark(super::journey::STEP_HELP);
}

/// Prints arguments separated by spaces.
pub fn cmd_echo(args: &[&str]) {
    let mut first = true;
    for arg in args {
        if !first {
            minios_hal::print!(" ");
        }
        minios_hal::print!("{}", arg);
        first = false;
    }
    println!();
}

/// Clears the screen.
pub fn cmd_clear(_args: &[&str]) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(ref mut console) = *minios_hal::framebuffer::CONSOLE.lock() {
            console.clear();
        }
    });
}

/// Shows uptime in human-readable minutes and seconds.
pub fn cmd_uptime(_args: &[&str]) {
    let ticks = minios_hal::interrupts::tick_count();
    let seconds = ticks / 100;
    let minutes = seconds / 60;
    println!("Uptime: {}m {}s ({} ticks)", minutes, seconds % 60, ticks);
    super::journey::mark(super::journey::STEP_UPTIME);
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

/// Lists the command history buffer.
pub fn cmd_history(_args: &[&str]) {
    let hist = crate::shell::HISTORY.lock();
    let count = hist.len();
    for i in 0..count {
        if let Some(entry) = hist.get(i) {
            println!("  {}  {}", i + 1, entry);
        }
    }
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

/// Controls the kernel log system.
pub fn cmd_log(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: log <level|module|history|off> [value]");
        println!("  log level <error|warn|info|debug|trace>");
        println!("  log module <name|all>");
        println!("  log history [count]");
        println!("  log off");
        println!(
            "Current: level={}, module=all",
            minios_hal::log::current_level().as_str()
        );
        return;
    }
    match args[0] {
        "level" => {
            if args.len() < 2 {
                println!("Current: {}", minios_hal::log::current_level().as_str());
                return;
            }
            match minios_hal::log::LogLevel::from_str(args[1]) {
                Some(l) => {
                    minios_hal::log::set_level(l);
                    println!("Log level: {}", l.as_str());
                }
                None => println!("Unknown level. Use: error, warn, info, debug, trace"),
            }
        }
        "module" => {
            if args.len() < 2 {
                println!("Usage: log module <name|all>");
                return;
            }
            minios_hal::log::set_module_filter(args[1]);
            println!("Log module filter: {}", args[1]);
        }
        "history" => {
            let count = if args.len() > 1 {
                args[1]
                    .bytes()
                    .fold(0usize, |acc, b| {
                        if b.is_ascii_digit() {
                            acc * 10 + (b - b'0') as usize
                        } else {
                            acc
                        }
                    })
                    .max(1)
            } else {
                20
            };
            let entries = minios_hal::log::recent_logs(count);
            for e in &entries {
                println!(
                    "[{}] [{}] {}",
                    e.level.as_str(),
                    e.module_str(),
                    e.message_str()
                );
            }
            if entries.is_empty() {
                println!("(no log entries)");
            }
        }
        "off" => {
            minios_hal::log::set_level(minios_hal::log::LogLevel::Error);
            println!("Logging minimized (errors only).");
        }
        _ => println!("Unknown log subcommand. Try: level, module, history, off"),
    }
}

/// Toggles debug mode (trace-level logging for all modules).
pub fn cmd_debug(args: &[&str]) {
    if args.is_empty() || args[0] == "status" {
        println!(
            "Debug mode: log level = {}",
            minios_hal::log::current_level().as_str()
        );
        return;
    }
    match args[0] {
        "on" => {
            minios_hal::log::set_level(minios_hal::log::LogLevel::Trace);
            minios_hal::log::set_module_filter("all");
            println!("Debug mode ON — all logs visible (level=TRACE, module=all)");
        }
        "off" => {
            minios_hal::log::set_level(minios_hal::log::LogLevel::Info);
            minios_hal::log::set_module_filter("all");
            println!("Debug mode OFF — normal logging (level=INFO)");
        }
        _ => println!("Usage: debug <on|off|status>"),
    }
}

/// Displays MiniOS version and system information.
pub fn cmd_version(_args: &[&str]) {
    println!("MiniOS v0.26.0");
    println!("Architecture: x86-64 (bare metal)");
    println!("Shell commands: {}", super::list_commands().len());
    println!("Subsystems: HAL, Trace, Memory, Process, Scheduler, FS, IPC, Syscall, Shell");
    println!("Tests: 105+");
    println!("Build: Rust nightly, bootloader_api 0.11");
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

/// Runs a command once for each item in a list.
pub fn cmd_each(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: each <command> <args...>");
        return;
    }
    let cmd = args[0];
    for item in &args[1..] {
        println!("> {} {}", cmd, item);
        let line = alloc::format!("{} {}", cmd, item);
        let parsed = crate::parser::parse(&line);
        if !parsed.is_empty() {
            if let Some(command) = crate::commands::find_command(parsed.command()) {
                (command.handler)(parsed.args());
            }
        }
    }
}

/// Repeats a command N times.
pub fn cmd_repeat(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: repeat <N> <command> [args...]");
        return;
    }
    let n = parse_usize_basic(args[0]).unwrap_or(1).min(100);
    let cmd_line: alloc::string::String = args[1..].join(" ");
    for i in 0..n {
        println!("--- iteration {} ---", i + 1);
        let parsed = crate::parser::parse(&cmd_line);
        if !parsed.is_empty() {
            if let Some(command) = crate::commands::find_command(parsed.command()) {
                (command.handler)(parsed.args());
            }
        }
    }
}

fn parse_usize_basic(s: &str) -> Option<usize> {
    let mut r: usize = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        r = r.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(r)
}

/// Reads a file and executes each line as a shell command.
pub fn cmd_run(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: run <script_file>");
        return;
    }

    use minios_common::traits::fs::FileSystem;
    use minios_common::types::OpenFlags;

    let path = args[0];
    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("run: filesystem not initialized");
            return;
        }
    };

    let fd = match vfs.open(path, OpenFlags::READ) {
        Ok(fd) => fd,
        Err(e) => {
            println!("run: {}: {}", path, e);
            return;
        }
    };

    let mut buf = [0u8; 2048];
    let n = match vfs.read(fd, &mut buf) {
        Ok(n) => n,
        Err(e) => {
            println!("run: read error: {}", e);
            return;
        }
    };
    vfs.close(fd).ok();
    drop(vfs_guard);

    let content = match core::str::from_utf8(&buf[..n]) {
        Ok(s) => s,
        Err(_) => {
            println!("run: file is not valid UTF-8");
            return;
        }
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        println!("> {}", line);
        let parsed = crate::parser::parse(line);
        if parsed.is_empty() {
            continue;
        }
        let cmd_name = parsed.command();
        let cmd_args = parsed.args();
        match crate::commands::find_command(cmd_name) {
            Some(command) => (command.handler)(cmd_args),
            None => println!("run: unknown command: {}", cmd_name),
        }
    }
    super::journey::mark(super::journey::STEP_RUN);
}

/// Exports a summary of the current learning session.
pub fn cmd_export_session(_args: &[&str]) {
    println!("=== Session Export ===");
    let hist_count = crate::shell::HISTORY.lock().len();
    println!("Commands executed: {}", hist_count);
    for i in 0..hist_count {
        if let Some(cmd) = crate::shell::HISTORY.lock().get(i) {
            println!("  {}. {}", i + 1, cmd);
        }
    }
    println!();
    let completed = crate::commands::journey::completed_count();
    println!("Journey progress: {}/17 steps", completed);
    println!("---");
}

/// Prints a quick reference card of all MiniOS command categories.
pub fn cmd_cheatsheet(_args: &[&str]) {
    println!("+---------------------------------------+");
    println!("|     MiniOS Quick Reference Card       |");
    println!("+---------------------------------------+");
    println!("| LEARN    tutorial explain compare lab |");
    println!("| PROCESS  ps spawn kill signal sched   |");
    println!("| MEMORY   meminfo frames pagetable     |");
    println!("| FILES    ls cat mkdir write touch rm   |");
    println!("| TRACE    trace follow/tree/filter/log  |");
    println!("| TEST     crash lab bench               |");
    println!("| SCRIPT   run each repeat alias history |");
    println!("| TEXT     head grep wc                   |");
    println!("| STATUS   top uptime interrupts version |");
    println!("| HELP     help man explain cheatsheet   |");
    println!("+---------------------------------------+");
}

/// Answers common learner questions about MiniOS.
pub fn cmd_faq(_args: &[&str]) {
    println!("=== Frequently Asked Questions ===");
    println!();
    println!("Q: How do I start learning?");
    println!("A: Type 'tutorial' for a guided 10-step walkthrough.");
    println!();
    println!("Q: What does 'explain' do?");
    println!("A: Shows how a command works internally without running it.");
    println!();
    println!("Q: Can I break the system?");
    println!("A: Yes! 'crash oom' safely demonstrates failures. Try it!");
    println!();
    println!("Q: How do I see what the OS is doing internally?");
    println!("A: 'trace follow <cmd>' shows every system call in a command.");
    println!("   'log level debug' enables detailed kernel logging.");
    println!();
    println!("Q: How do I track my progress?");
    println!("A: 'journey' shows your learning path. 'graduation' for final report.");
    println!();
    println!("Q: Is this a real operating system?");
    println!("A: Yes! It boots on real x86-64 hardware (via QEMU). It has real");
    println!("   memory management, process scheduling, and a filesystem.");
}

/// Prints a structured course outline for using MiniOS as a teaching tool.
pub fn cmd_syllabus(_args: &[&str]) {
    println!("=== MiniOS Operating Systems Syllabus ===");
    println!();
    println!("Module 1: Process Management (2 hours)");
    println!("  Concepts: PCB, state machine, scheduling algorithms");
    println!("  Commands: ps, spawn, kill, signal, sched, compare scheduler");
    println!("  Lab: lab scheduler-fairness");
    println!("  Reading: explain ps, explain spawn, explain sched");
    println!();
    println!("Module 2: Memory Management (2 hours)");
    println!("  Concepts: Physical frames, virtual pages, heap allocation");
    println!("  Commands: meminfo, frames, pagetable, alloc");
    println!("  Lab: lab memory-usage, lab page-table-walk");
    println!("  Reading: explain meminfo, explain frames, explain pagetable");
    println!();
    println!("Module 3: File Systems (1.5 hours)");
    println!("  Concepts: VFS, inodes, file descriptors, directory tree");
    println!("  Commands: ls, cat, mkdir, write, touch, rm");
    println!("  Lab: lab fs-operations");
    println!("  Reading: explain ls, explain cat, compare filesystem");
    println!();
    println!("Module 4: System Calls & IPC (1.5 hours)");
    println!("  Concepts: Syscall interface, message passing, tracing");
    println!("  Commands: trace follow, trace tree, log, compare syscall/ipc");
    println!("  Lab: lab trace-overhead");
    println!("  Reading: explain trace, compare syscall, compare ipc");
    println!();
    println!("Module 5: Fault Handling (1 hour)");
    println!("  Concepts: Interrupts, exceptions, OOM, stack overflow");
    println!("  Commands: crash oom, crash stack, crash divide-zero, interrupts");
    println!("  Reading: explain log");
    println!();
    println!("Total: ~8 hours of guided hands-on learning");
}
