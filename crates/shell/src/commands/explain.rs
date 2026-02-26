//! The `explain` command — teaches what a command does without executing it.

use minios_hal::println;

pub fn cmd_explain(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: explain <command>");
        println!("Shows what a command does internally, without executing it.");
        return;
    }
    match args[0] {
        "ls" => explain_ls(),
        "cat" => explain_cat(),
        "ps" => explain_ps(),
        "meminfo" => explain_meminfo(),
        "trace" => explain_trace(),
        "spawn" => explain_spawn(),
        "sched" => explain_sched(),
        "pagetable" => explain_pagetable(),
        "frames" => explain_frames(),
        other => println!("No explanation available for '{}'.", other),
    }
}

fn explain_ls() {
    println!("=== How 'ls' works ===");
    println!();
    println!("1. Shell parses input → command='ls', args=[path]");
    println!("2. VFS resolves the path by walking the inode tree:");
    println!("   '/' → inode 0 (root) → lookup each path component");
    println!("3. For each child inode in the directory:");
    println!("   - Read the inode's type (file/dir) and size");
    println!("   - Format and print: type, size, name");
    println!();
    println!("Key concepts:");
    println!("  Inode  — a numbered record storing file metadata + data");
    println!("  VFS    — Virtual File System, abstracts storage backends");
    println!("  RamFS  — our in-memory filesystem (data lives in heap)");
}

fn explain_cat() {
    println!("=== How 'cat' works ===");
    println!();
    println!("1. sys_open(path, READ) → VFS finds the inode, creates a");
    println!("   file descriptor (FD) pointing to offset 0");
    println!("2. sys_read(fd, buf, 512) → VFS reads data from the inode");
    println!("   starting at the current offset, advances offset");
    println!("3. Print the bytes to screen, repeat until read returns 0");
    println!("4. sys_close(fd) → VFS releases the file descriptor");
    println!();
    println!("Key concepts:");
    println!("  File descriptor — a process-local handle to an open file");
    println!("  Offset — current read/write position within the file");
    println!("  For /proc/* files, content is generated on each open()");
}

fn explain_ps() {
    println!("=== How 'ps' works ===");
    println!();
    println!("1. Lock the process table (a fixed-size array of PCBs)");
    println!("2. For each non-empty slot, read:");
    println!("   - PID (Process ID, monotonically increasing)");
    println!("   - State (CREATED/READY/RUNNING/BLOCKED/TERMINATED)");
    println!("   - Priority (0=HIGH, 1=MED, 2=LOW, 3=IDLE)");
    println!("   - CPU time (ticks spent running)");
    println!();
    println!("Key concepts:");
    println!("  PCB — Process Control Block, stores all process state");
    println!("  State machine — processes transition through defined states");
    println!("  Scheduler decides which READY process runs next");
}

fn explain_meminfo() {
    println!("=== How 'meminfo' works ===");
    println!();
    println!("Memory has two layers:");
    println!("  Physical frames — 4 KiB blocks managed by a bitmap allocator");
    println!("    Each bit = 1 frame. Set = in use, clear = free.");
    println!("  Heap — contiguous virtual memory for alloc/Vec/String");
    println!("    Managed by a linked-list free-block allocator.");
    println!();
    println!("meminfo reads counters from both allocators.");
}

fn explain_trace() {
    println!("=== How tracing works ===");
    println!();
    println!("Every kernel operation can create a 'span':");
    println!("  begin_span(name, module) → push to context stack");
    println!("  ... do work ...");
    println!("  end_span() → pop context, record duration");
    println!();
    println!("Spans form a tree via parent_span_id linkage.");
    println!("The ring buffer stores the last 4096 spans.");
    println!("'trace follow <cmd>' clears the buffer, runs the");
    println!("command, then shows only that command's spans.");
}

fn explain_spawn() {
    println!("=== How 'spawn' works ===");
    println!();
    println!("1. Allocate a 16 KiB kernel stack from the heap");
    println!("2. Create a CpuContext: RSP=stack top, RIP=entry function");
    println!("3. Insert a PCB into the process table (next free slot)");
    println!("4. Add the PID to the scheduler's ready queue");
    println!();
    println!("The task won't actually run until the scheduler switches to it.");
    println!("Currently MiniOS uses cooperative scheduling (no preemption).");
}

fn explain_sched() {
    println!("=== How the MLFQ scheduler works ===");
    println!();
    println!("4 priority queues, each with a time slice:");
    println!("  Queue 0 [HIGH]:  2 ticks");
    println!("  Queue 1 [MED]:   4 ticks");
    println!("  Queue 2 [LOW]:   8 ticks");
    println!("  Queue 3 [IDLE]: 16 ticks");
    println!();
    println!("Rules:");
    println!("  - New tasks start at the highest priority");
    println!("  - Exhaust your time slice → demoted to next queue");
    println!("  - Every 100 ticks → all tasks boosted to queue 0");
    println!("  - Boost prevents starvation of low-priority tasks");
}

fn explain_pagetable() {
    println!("=== How virtual address translation works ===");
    println!();
    println!("x86-64 uses 4-level page tables (48-bit virtual addresses):");
    println!("  Bits 47-39: PML4 index (512 entries)");
    println!("  Bits 38-30: PDPT index (512 entries)");
    println!("  Bits 29-21: PD index   (512 entries)");
    println!("  Bits 20-12: PT index   (512 entries)");
    println!("  Bits 11-0:  Page offset (4096 bytes)");
    println!();
    println!("Each level is a table of 512 8-byte entries.");
    println!("Each entry contains the physical address of the next table");
    println!("(or the final frame) plus permission flags (Present, Writable).");
}

fn explain_frames() {
    println!("=== How physical frame allocation works ===");
    println!();
    println!("The bitmap allocator tracks every 4 KiB physical frame.");
    println!("  256 MiB RAM = 65,536 frames = 8,192 bytes of bitmap.");
    println!("  Bit set = frame in use, bit clear = frame free.");
    println!();
    println!("allocate_frame(): scan bitmap for first clear bit, set it.");
    println!("deallocate_frame(): check bit is set, then clear it.");
    println!("  (Double-free is detected and returns an error.)");
}
