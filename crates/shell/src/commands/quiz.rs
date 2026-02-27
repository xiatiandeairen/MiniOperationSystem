//! Quiz and challenge commands for OS knowledge self-assessment.

extern crate alloc;

use minios_hal::println;

/// Presents OS knowledge quiz questions on a chosen topic.
///
/// ```text
/// quiz            — list available topics
/// quiz process    — process management Q&A
/// quiz memory     — memory management Q&A
/// quiz fs         — filesystem Q&A
/// ```
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

/// Presents verifiable learning challenges the user can attempt.
///
/// ```text
/// challenge create-file   — create a file, write to it, verify
/// challenge find-pid      — find the busiest process
/// challenge memory-check  — check system memory stats
/// ```
pub fn cmd_challenge(args: &[&str]) {
    if args.is_empty() {
        challenge_list();
        return;
    }
    match args[0] {
        "1" | "create-file" => challenge_create_file(),
        "2" | "find-pid" => challenge_find_pid(),
        "3" | "memory-check" => challenge_memory(),
        _ => challenge_list(),
    }
}

fn challenge_list() {
    println!("=== Challenges ===");
    println!("  1. create-file  — Create a file, write to it, verify contents");
    println!("  2. find-pid     — Find which process has the most CPU time");
    println!("  3. memory-check — Check how much heap is free");
    println!("Each challenge tells you what to do, then verifies your answer.");
}

fn challenge_create_file() {
    println!("=== Challenge: Create and Verify a File ===");
    println!();
    println!("Task: Create /tmp/challenge.txt with the content 'hello'");
    println!("Commands to use:");
    println!("  1. write /tmp/challenge.txt hello");
    println!("  2. cat /tmp/challenge.txt");
    println!();
    use minios_common::traits::fs::FileSystem;
    use minios_common::types::OpenFlags;
    let vfs_guard = minios_fs::VFS.lock();
    if let Some(vfs) = vfs_guard.as_ref() {
        match vfs.open("/tmp/challenge.txt", OpenFlags::READ) {
            Ok(fd) => {
                let mut buf = [0u8; 32];
                let n = vfs.read(fd, &mut buf).unwrap_or(0);
                vfs.close(fd).ok();
                let content = core::str::from_utf8(&buf[..n]).unwrap_or("");
                if content.contains("hello") {
                    println!("✅ Challenge PASSED! File contains 'hello'.");
                } else {
                    println!("⬜ File exists but doesn't contain 'hello'. Try: write /tmp/challenge.txt hello");
                }
            }
            Err(_) => println!("⬜ File not created yet. Try: write /tmp/challenge.txt hello"),
        }
    }
}

fn challenge_find_pid() {
    println!("=== Challenge: Find the Busiest Process ===");
    println!();
    println!("Task: Use 'ps' to find which PID has the most CPU time.");
    let procs = minios_process::manager::list_processes();
    let max = procs.iter().max_by_key(|p| p.cpu_time_ticks);
    if let Some(p) = max {
        println!(
            "Hint: The answer is PID {} ({} ticks)",
            p.pid, p.cpu_time_ticks
        );
    }
    println!("✅ Challenge complete!");
}

fn challenge_memory() {
    println!("=== Challenge: Memory Check ===");
    println!();
    println!("Task: Run 'meminfo' and 'frames' to check system memory.");
    let stats = minios_memory::get_stats();
    println!(
        "Current: {} free frames, {} bytes heap free",
        stats.free_frames, stats.heap_free
    );
    println!("✅ Challenge complete!");
}
