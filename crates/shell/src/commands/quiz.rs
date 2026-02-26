//! Quiz and challenge commands for OS knowledge self-assessment.

use minios_hal::println;

pub fn cmd_quiz(args: &[&str]) {
    let topic = if args.is_empty() { "all" } else { args[0] };
    match topic {
        "process" | "1" => quiz_process(),
        "memory" | "2" => quiz_memory(),
        "fs" | "3" => quiz_fs(),
        _ => quiz_list(),
    }
}

fn quiz_list() {
    println!("=== OS Knowledge Quiz ===");
    println!("  1. process  — Process management questions");
    println!("  2. memory   — Memory management questions");
    println!("  3. fs       — Filesystem questions");
    println!("Usage: quiz <topic>");
}

fn quiz_process() {
    println!("=== Quiz: Process Management ===");
    println!();
    println!("Q1: What data structure stores a process's state?");
    println!("    Answer: PCB (Process Control Block)");
    println!();
    println!("Q2: What are the 5 process states in MiniOS?");
    println!("    Answer: Created, Ready, Running, Blocked, Terminated");
    println!();
    println!("Q3: How does MLFQ prevent starvation?");
    println!("    Answer: Periodic priority boost moves all tasks to the highest queue.");
    println!();
    println!("Verify with: ps, explain ps, compare scheduler");
}

fn quiz_memory() {
    println!("=== Quiz: Memory Management ===");
    println!();
    println!("Q1: How many levels are in an x86-64 page table?");
    println!("    Answer: 4 (PML4 → PDPT → PD → PT)");
    println!();
    println!("Q2: What is the size of one physical frame?");
    println!("    Answer: 4 KiB (4096 bytes)");
    println!();
    println!("Q3: What happens when the heap runs out of memory?");
    println!("    Answer: MiniOS panics. Linux's OOM killer terminates a process.");
    println!();
    println!("Verify with: meminfo, pagetable, crash oom");
}

fn quiz_fs() {
    println!("=== Quiz: Filesystem ===");
    println!();
    println!("Q1: What does VFS stand for?");
    println!("    Answer: Virtual File System — abstracts storage backends");
    println!();
    println!("Q2: What is an inode?");
    println!("    Answer: A numbered record storing file metadata and data");
    println!();
    println!("Q3: What is a file descriptor?");
    println!("    Answer: A process-local integer handle to an open file");
    println!();
    println!("Verify with: ls, cat, explain ls, explain cat");
}
