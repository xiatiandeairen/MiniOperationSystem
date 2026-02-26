//! Learning journey — tracks progress through MiniOS exploration.

use core::sync::atomic::{AtomicU32, Ordering};
use minios_hal::println;

/// Bitmap tracking completed journey steps (up to 32 steps).
static COMPLETED: AtomicU32 = AtomicU32::new(0);

// Chapter 1: First Steps
pub const STEP_HELP: u32 = 0;
pub const STEP_PS: u32 = 1;
pub const STEP_MEMINFO: u32 = 2;
pub const STEP_LS: u32 = 3;
pub const STEP_UPTIME: u32 = 4;

// Chapter 2: Understanding the Internals
pub const STEP_EXPLAIN: u32 = 5;
pub const STEP_CAT_PROC: u32 = 6;
pub const STEP_TRACE_FOLLOW: u32 = 7;
pub const STEP_PAGETABLE: u32 = 8;

// Chapter 3: Experiments
pub const STEP_LAB: u32 = 9;
pub const STEP_CRASH: u32 = 10;
pub const STEP_COMPARE: u32 = 11;
pub const STEP_FRAMES: u32 = 12;

// Chapter 4: Building & Combining
pub const STEP_SPAWN: u32 = 13;
pub const STEP_RUN: u32 = 14;
pub const STEP_GREP: u32 = 15;
pub const STEP_TRACE_STATS: u32 = 16;

const TOTAL_STEPS: u32 = 17;

/// Marks a journey step as complete.
pub fn mark(step: u32) {
    COMPLETED.fetch_or(1 << step, Ordering::Relaxed);
}

fn is_done(step: u32) -> bool {
    COMPLETED.load(Ordering::Relaxed) & (1 << step) != 0
}

fn chapter_progress(steps: &[u32]) -> (usize, usize) {
    let done = steps.iter().filter(|&&s| is_done(s)).count();
    (done, steps.len())
}

fn step_marker(step: u32) -> &'static str {
    if is_done(step) {
        "[x]"
    } else {
        "[ ]"
    }
}

fn print_chapter(title: &str, steps: &[(u32, &str, &str)]) {
    let ids: [u32; 5] = {
        let mut a = [0u32; 5];
        for (i, &(id, _, _)) in steps.iter().enumerate() {
            if i < 5 {
                a[i] = id;
            }
        }
        a
    };
    let (done, total) = chapter_progress(&ids[..steps.len()]);
    println!("{} ({}/{})", title, done, total);
    for &(id, cmd, desc) in steps {
        println!("  {} {:16} {}", step_marker(id), cmd, desc);
    }
    println!();
}

/// Shows the learning journey progress across all chapters.
pub fn cmd_journey(_args: &[&str]) {
    println!("=== Your MiniOS Learning Journey ===");
    println!();

    print_chapter(
        "Chapter 1: First Steps",
        &[
            (STEP_HELP, "help", "see all commands"),
            (STEP_PS, "ps", "view processes"),
            (STEP_MEMINFO, "meminfo", "check memory"),
            (STEP_LS, "ls /", "browse filesystem"),
            (STEP_UPTIME, "uptime", "check system time"),
        ],
    );

    print_chapter(
        "Chapter 2: Understanding the Internals",
        &[
            (STEP_EXPLAIN, "explain <cmd>", "learn how commands work"),
            (STEP_CAT_PROC, "cat /proc/*", "read virtual files"),
            (STEP_TRACE_FOLLOW, "trace follow <cmd>", "trace a command"),
            (STEP_PAGETABLE, "pagetable <addr>", "explore page tables"),
        ],
    );

    print_chapter(
        "Chapter 3: Experiments",
        &[
            (STEP_LAB, "lab <name>", "run interactive labs"),
            (STEP_CRASH, "crash <scenario>", "safe fault experiments"),
            (STEP_COMPARE, "compare <topic>", "MiniOS vs Linux"),
            (STEP_FRAMES, "frames", "visualize frame usage"),
        ],
    );

    print_chapter(
        "Chapter 4: Building & Combining",
        &[
            (STEP_SPAWN, "spawn <name>", "create a process"),
            (STEP_RUN, "run <script>", "execute a script file"),
            (STEP_GREP, "grep <pat> <file>", "search in files"),
            (STEP_TRACE_STATS, "trace stats", "view trace statistics"),
        ],
    );

    let total_done = (0..TOTAL_STEPS).filter(|&i| is_done(i)).count();
    let pct = total_done * 100 / TOTAL_STEPS as usize;
    println!(
        "Overall: {}/{} steps complete ({}%)",
        total_done, TOTAL_STEPS, pct
    );

    if total_done == 0 {
        println!("Start with: help");
    } else if total_done < TOTAL_STEPS as usize {
        // Find next incomplete step
        for i in 0..TOTAL_STEPS {
            if !is_done(i) {
                let hint = match i {
                    0 => "Try: help",
                    1 => "Try: ps",
                    2 => "Try: meminfo",
                    3 => "Try: ls /",
                    4 => "Try: uptime",
                    5 => "Try: explain ls",
                    6 => "Try: cat /proc/uptime",
                    7 => "Try: trace follow ls /",
                    8 => "Try: pagetable 0x4444_4444_0000",
                    9 => "Try: lab scheduler-fairness",
                    10 => "Try: crash oom",
                    11 => "Try: compare scheduler",
                    12 => "Try: frames",
                    13 => "Try: spawn worker",
                    14 => "Try: run /etc/init.sh",
                    15 => "Try: grep <pattern> <file>",
                    16 => "Try: trace stats",
                    _ => "",
                };
                if !hint.is_empty() {
                    println!("Next: {}", hint);
                }
                break;
            }
        }
    } else {
        println!("All steps complete! Run 'graduation' to see your report.");
    }
}

/// Outputs a structured learning progress report.
pub fn cmd_report(_args: &[&str]) {
    let completed = COMPLETED.load(Ordering::Relaxed);
    let total = 17u32;
    let done = (0..total).filter(|&i| completed & (1 << i) != 0).count();

    println!("--- Learning Report ---");
    println!("student: MiniOS User");
    println!("date: (current session)");
    println!(
        "progress: {}/{} steps ({}%)",
        done,
        total,
        done * 100 / total as usize
    );
    println!("chapters_completed:");
    println!("concepts_learned:");
    if completed & (1 << 1) != 0 {
        println!("  - process management");
    }
    if completed & (1 << 2) != 0 {
        println!("  - memory management");
    }
    if completed & (1 << 3) != 0 {
        println!("  - filesystem operations");
    }
    if completed & (1 << 5) != 0 {
        println!("  - system internals (explain)");
    }
    if completed & (1 << 9) != 0 {
        println!("  - hands-on experiments (lab)");
    }
    if completed & (1 << 10) != 0 {
        println!("  - fault handling (crash)");
    }
    if completed & (1 << 11) != 0 {
        println!("  - OS design comparisons");
    }
    println!("---");
}

/// Shows the graduation/completion report.
pub fn cmd_graduation(_args: &[&str]) {
    let done = (0..TOTAL_STEPS).filter(|&i| is_done(i)).count();
    let pct = done * 100 / TOTAL_STEPS as usize;

    if pct < 50 {
        println!(
            "You've completed {}% of the journey ({}/{}).",
            pct, done, TOTAL_STEPS
        );
        println!("Keep exploring! Type 'journey' to see your progress.");
        return;
    }

    if pct < 100 {
        println!(
            "Great progress! {}% complete ({}/{})",
            pct, done, TOTAL_STEPS
        );
        println!("Type 'journey' to see remaining steps.");
        return;
    }

    println!("========================================");
    println!("  Congratulations!");
    println!("  You completed the MiniOS learning journey!");
    println!("========================================");
    println!();
    println!("OS concepts you explored:");
    println!("  [x] Process management    — ps, spawn, kill, signal, scheduler");
    println!("  [x] Memory management     — meminfo, frames, pagetable, heap alloc");
    println!("  [x] Filesystem            — ls, cat, mkdir, write, VFS + RamFS + ProcFS");
    println!("  [x] Tracing               — trace follow, trace tree, trace export");
    println!("  [x] Fault handling         — crash oom, stack overflow, page faults");
    println!("  [x] OS design trade-offs   — compare scheduler/memory/fs/ipc/syscall");
    println!();
    println!(
        "You completed {}/{} learning steps across 4 chapters.",
        done, TOTAL_STEPS
    );
    println!("Well done!");
}
