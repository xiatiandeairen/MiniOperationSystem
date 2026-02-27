//! The `compare` command — shows MiniOS vs industry OS design differences.

use minios_hal::println;

/// Compares MiniOS design choices against industry operating systems.
///
/// ```text
/// compare scheduler   — MLFQ vs CFS
/// compare memory      — bitmap vs buddy system
/// compare filesystem  — RamFS vs ext4
/// compare ipc         — message queue vs pipes/sockets/shm
/// compare syscall     — direct call vs int 0x80
/// ```
pub fn cmd_compare(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: compare <topic>");
        println!("Topics: scheduler, memory, filesystem, ipc, syscall");
        return;
    }
    match args[0] {
        "scheduler" | "sched" => compare_scheduler(),
        "memory" | "mem" => compare_memory(),
        "filesystem" | "fs" => compare_filesystem(),
        "ipc" => compare_ipc(),
        "syscall" => compare_syscall(),
        _ => {
            println!("Unknown topic. Try: scheduler, memory, filesystem, ipc, syscall");
            return;
        }
    }
    super::journey::mark(super::journey::STEP_COMPARE);
}

fn compare_scheduler() {
    println!("=== Scheduler: MiniOS vs Linux ===");
    println!();
    println!("  {:16} {:20} {:20}", "Aspect", "MiniOS", "Linux");
    println!("  {:16} {:20} {:20}", "------", "------", "-----");
    println!(
        "  {:16} {:20} {:20}",
        "Algorithm", "MLFQ (4 levels)", "CFS (red-black tree)"
    );
    println!(
        "  {:16} {:20} {:20}",
        "Time slice", "Fixed 2/4/8/16", "Dynamic (vruntime)"
    );
    println!(
        "  {:16} {:20} {:20}",
        "Preemption", "Cooperative", "Fully preemptive"
    );
    println!(
        "  {:16} {:20} {:20}",
        "SMP", "Single core", "Per-CPU runqueues"
    );
    println!("  {:16} {:20} {:20}", "Real-time", "None", "SCHED_FIFO/RR");
    println!(
        "  {:16} {:20} {:20}",
        "Fairness", "Priority boost", "Virtual runtime"
    );
    println!();
    println!("Why does Linux use CFS instead of MLFQ?");
    println!("  CFS automatically ensures fairness via virtual runtime tracking.");
    println!("  MLFQ requires manual tuning of time slices and boost intervals.");
    println!("  But MLFQ is simpler to understand — ideal for learning.");
}

fn compare_memory() {
    println!("=== Memory: MiniOS vs Linux ===");
    println!();
    println!("  {:16} {:20} {:20}", "Aspect", "MiniOS", "Linux");
    println!("  {:16} {:20} {:20}", "------", "------", "-----");
    println!(
        "  {:16} {:20} {:20}",
        "Frame alloc", "Bitmap", "Buddy system"
    );
    println!("  {:16} {:20} {:20}", "Page size", "4 KiB only", "4K+2M+1G");
    println!("  {:16} {:20} {:20}", "Heap", "Linked list", "Slab+SLUB");
    println!("  {:16} {:20} {:20}", "NUMA", "No", "Yes");
    println!("  {:16} {:20} {:20}", "Swap", "No", "Yes");
    println!("  {:16} {:20} {:20}", "OOM", "Panic", "OOM killer");
    println!();
    println!("Why does Linux use a buddy system instead of a bitmap?");
    println!("  The buddy system merges adjacent free blocks efficiently,");
    println!("  reducing fragmentation. Bitmap search is O(n) in the worst case.");
    println!("  But a bitmap is simpler to implement — ideal for learning.");
}

fn compare_filesystem() {
    println!("=== Filesystem: MiniOS vs Linux ===");
    println!();
    println!("  {:16} {:20} {:20}", "Aspect", "MiniOS", "Linux");
    println!("  {:16} {:20} {:20}", "------", "------", "-----");
    println!(
        "  {:16} {:20} {:20}",
        "VFS", "Trait-based", "VFS + inode cache"
    );
    println!(
        "  {:16} {:20} {:20}",
        "Storage", "RAM only", "Disk (ext4, btrfs..)"
    );
    println!("  {:16} {:20} {:20}", "Journaling", "No", "Yes");
    println!("  {:16} {:20} {:20}", "Permissions", "No", "rwx + ACL");
    println!("  {:16} {:20} {:20}", "Mount", "Single root", "Mount table");
    println!(
        "  {:16} {:20} {:20}",
        "Max file size", "Heap limit", "16 TiB (ext4)"
    );
    println!();
    println!("Why does Linux use journaling?");
    println!("  Journaling prevents data corruption on unexpected power loss.");
    println!("  MiniOS lives in RAM, so data is lost on reboot anyway.");
    println!("  But understanding journaling is key to real filesystem design.");
}

fn compare_ipc() {
    println!("=== IPC: MiniOS vs Linux ===");
    println!();
    println!("  {:16} {:20} {:20}", "Aspect", "MiniOS", "Linux");
    println!("  {:16} {:20} {:20}", "------", "------", "-----");
    println!(
        "  {:16} {:20} {:20}",
        "Mechanism", "Message queue", "pipe, socket, shm, mq"
    );
    println!(
        "  {:16} {:20} {:20}",
        "Max size", "256 bytes", "Configurable"
    );
    println!("  {:16} {:20} {:20}", "Blocking", "No", "Yes");
    println!("  {:16} {:20} {:20}", "Shared mem", "No", "Yes");
    println!("  {:16} {:20} {:20}", "Signals", "No", "31+ signals");
    println!("  {:16} {:20} {:20}", "Sockets", "No", "Yes (TCP/UDP)");
    println!();
    println!("Why does Linux need so many IPC mechanisms?");
    println!("  Different use cases need different trade-offs:");
    println!("  pipes for streaming, shared memory for speed, sockets for network.");
    println!("  MiniOS uses a simple message queue to teach the core concept.");
}

fn compare_syscall() {
    println!("=== Syscall: MiniOS vs Linux ===");
    println!();
    println!("  {:16} {:20} {:20}", "Aspect", "MiniOS", "Linux");
    println!("  {:16} {:20} {:20}", "------", "------", "-----");
    println!(
        "  {:16} {:20} {:20}",
        "Mechanism", "Function call", "int 0x80/syscall"
    );
    println!("  {:16} {:20} {:20}", "Count", "7", "400+");
    println!("  {:16} {:20} {:20}", "Entry", "Direct call", "IDT/MSR");
    println!("  {:16} {:20} {:20}", "Validation", "Minimal", "Extensive");
    println!(
        "  {:16} {:20} {:20}",
        "Tracing", "trace_span! macro", "ftrace/strace"
    );
    println!();
    println!("Why does Linux use int 0x80 / syscall instead of function calls?");
    println!("  The syscall instruction switches from user mode (ring 3) to");
    println!("  kernel mode (ring 0), enforcing protection boundaries.");
    println!("  MiniOS runs entirely in ring 0, so direct calls suffice.");
}
