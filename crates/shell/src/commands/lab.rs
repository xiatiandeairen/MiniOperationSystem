//! Built-in labs for hands-on OS learning.

extern crate alloc;

use minios_hal::println;

/// Runs interactive OS learning experiments.
///
/// ```text
/// lab list                 — show available labs
/// lab scheduler-fairness   — compare CPU time across priorities
/// lab memory-usage         — observe heap allocation effects
/// lab page-table-walk      — manually translate a virtual address
/// lab trace-overhead       — measure the cost of tracing
/// lab fs-operations        — create, write, read, delete a file
/// ```
pub fn cmd_lab(args: &[&str]) {
    if args.is_empty() {
        lab_list();
        return;
    }
    match args[0] {
        "list" => lab_list(),
        "scheduler-fairness" | "1" => lab_scheduler_fairness(),
        "memory-usage" | "2" => lab_memory_usage(),
        "page-table-walk" | "3" => lab_page_table_walk(),
        "trace-overhead" | "4" => lab_trace_overhead(),
        "fs-operations" | "5" => lab_fs_operations(),
        "scheduler-compare" | "6" => lab_scheduler_compare(),
        "allocator-compare" | "7" => lab_allocator_compare(),
        _ => println!("Unknown lab. Type 'lab list' to see available labs."),
    }
    if args[0] != "list" {
        super::journey::mark(super::journey::STEP_LAB);
    }
}

fn lab_list() {
    println!("=== Available Labs ===");
    println!();
    println!("  1. scheduler-fairness  — Compare CPU time across priorities");
    println!("  2. memory-usage        — Observe heap allocation effects");
    println!("  3. page-table-walk     — Manually translate a virtual address");
    println!("  4. trace-overhead      — Measure the cost of tracing");
    println!("  5. fs-operations       — Create, write, read, delete a file");
    println!("  6. scheduler-compare   — Compare MLFQ vs Round-Robin algorithms");
    println!("  7. allocator-compare   — Compare Bitmap vs Buddy System allocators");
    println!();
    println!("Run a lab: lab <name>  or  lab <number>");
}

fn lab_scheduler_fairness() {
    println!("=== Lab: Scheduler Fairness ===");
    println!();
    println!("This lab shows how MLFQ distributes CPU time across priorities.");
    println!();

    let sched = minios_scheduler::SCHEDULER.lock();
    let stats = sched.stats();
    drop(sched);

    let procs = minios_process::manager::list_processes();

    println!("Current process CPU time distribution:");
    println!("  {:>5} {:>10} {:>10}", "PID", "Priority", "CPU ticks");
    for p in &procs {
        println!(
            "  {:>5} {:>10} {:>10}",
            p.pid, p.priority.0, p.cpu_time_ticks
        );
    }
    println!();
    println!(
        "Scheduler totals: {} ticks, {} switches",
        stats.total_ticks, stats.total_switches
    );
    println!();
    println!("Observation: Higher priority tasks (lower number) get");
    println!("scheduled more frequently, accumulating more CPU time.");
    println!();
    println!("Try: 'spawn worker' then 'nice 2 3' to change its priority,");
    println!("then run 'lab scheduler-fairness' again to see the effect.");
    println!();
    println!("Thinking question: What happens without priority boost?");
    println!("  → Low-priority tasks would starve (never get CPU time).");
    println!("  → MiniOS boosts all tasks every 100 ticks to prevent this.");
    println!();
    println!("✅ Lab complete!");
}

fn lab_memory_usage() {
    println!("=== Lab: Memory Usage ===");
    println!();

    let before = minios_memory::get_stats();
    println!("Before allocation:");
    println!("  Heap used: {} bytes", before.heap_used);
    println!("  Heap free: {} bytes", before.heap_free);
    println!();

    println!("Allocating 10 x 1024 bytes...");
    let mut blocks: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
    for _ in 0..10 {
        blocks.push(alloc::vec![0u8; 1024]);
    }

    let after = minios_memory::get_stats();
    println!();
    println!("After allocation:");
    println!(
        "  Heap used: {} bytes (+{})",
        after.heap_used,
        after.heap_used - before.heap_used
    );
    println!("  Heap free: {} bytes", after.heap_free);
    println!();

    drop(blocks);
    let freed = minios_memory::get_stats();
    println!("After freeing:");
    println!("  Heap used: {} bytes", freed.heap_used);
    println!("  Heap free: {} bytes", freed.heap_free);
    println!();

    println!("Observation: The allocator tracks used/free precisely.");
    println!("After dropping, memory returns to approximately the same level.");
    println!("(Not exact due to allocator metadata overhead.)");
    println!();
    println!("Thinking question: Why might freed memory not match exactly?");
    println!("  → The linked-list allocator stores metadata in freed blocks.");
    println!("  → Fragmentation can prevent adjacent blocks from merging.");
    println!();
    println!("✅ Lab complete!");
}

fn lab_page_table_walk() {
    println!("=== Lab: Page Table Walk ===");
    println!();
    println!("Let's translate the heap start address: 0x4444_4444_0000");
    println!();

    let addr: u64 = 0x4444_4444_0000;
    let pml4 = (addr >> 39) & 0x1FF;
    let pdpt = (addr >> 30) & 0x1FF;
    let pd = (addr >> 21) & 0x1FF;
    let pt = (addr >> 12) & 0x1FF;
    let offset = addr & 0xFFF;

    println!("Virtual address: {:#018x}", addr);
    println!();
    println!("Step 1: PML4 index = {} (bits 47-39)", pml4);
    println!("  CR3 register points to the PML4 table (Level 4).");
    println!(
        "  Entry {} contains the physical address of the PDPT.",
        pml4
    );
    println!();
    println!("Step 2: PDPT index = {} (bits 38-30)", pdpt);
    println!("  This table is Level 3. Entry {} points to the PD.", pdpt);
    println!();
    println!("Step 3: PD index = {} (bits 29-21)", pd);
    println!("  This table is Level 2. Entry {} points to the PT.", pd);
    println!();
    println!("Step 4: PT index = {} (bits 20-12)", pt);
    println!(
        "  This table is Level 1. Entry {} contains the physical frame.",
        pt
    );
    println!();
    println!("Step 5: Offset = {} (bits 11-0)", offset);
    println!("  Added to the frame's base address to get the final byte.");
    println!();
    println!("Total: 4 table lookups + 1 offset addition = physical address.");
    println!();
    println!("Thinking question: Why 4 levels instead of 1 flat table?");
    println!("  → A flat table for 48-bit addresses = 256 TiB of entries!");
    println!("  → 4 levels allocate tables only for used regions (sparse).");
    println!();
    println!("✅ Lab complete!");
}

fn lab_trace_overhead() {
    use minios_common::traits::trace::Tracer;

    println!("=== Lab: Trace Overhead ===");
    println!();
    println!("Measuring the cost of creating trace spans...");
    println!();

    let start_tsc = minios_hal::cpu::read_tsc();

    for _ in 0..100 {
        let _span = minios_trace::trace_span!("bench", module = "lab");
    }

    let end_tsc = minios_hal::cpu::read_tsc();
    let total = end_tsc - start_tsc;
    let per_span = total / 100;

    println!("100 spans created + closed:");
    println!("  Total cycles: {}", total);
    println!("  Per span:     {} cycles", per_span);
    println!(
        "  Estimated:    ~{} ns (at 2.4 GHz)",
        per_span * 1000 / 2400
    );
    println!();

    let stats = minios_trace::TRACER.stats();
    println!(
        "Ring buffer: {}/{} slots used",
        stats.buffer_used, stats.buffer_capacity
    );
    println!();

    println!(
        "Observation: Each span costs ~{} cycles of overhead.",
        per_span
    );
    println!("This includes: atomic ID generation, TSC read, mutex lock,");
    println!("ring buffer write, context stack push/pop.");
    println!();
    println!("Thinking question: How could we reduce this overhead?");
    println!("  → Lock-free ring buffer (CAS instead of mutex)");
    println!("  → Per-CPU buffers (no contention)");
    println!("  → Compile-time trace disable (zero cost when off)");
    println!();
    println!("✅ Lab complete!");
}

fn lab_fs_operations() {
    use minios_common::traits::fs::FileSystem;
    use minios_common::types::OpenFlags;

    println!("=== Lab: Filesystem Operations ===");
    println!();

    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("Filesystem not initialized!");
            return;
        }
    };

    println!("Step 1: Create file /tmp/lab_test.txt");
    let fd = match vfs.open("/tmp/lab_test.txt", OpenFlags::CREATE | OpenFlags::WRITE) {
        Ok(fd) => fd,
        Err(e) => {
            println!("  Failed to create file: {:?}", e);
            return;
        }
    };
    println!("  → File descriptor: {}", fd.0);

    let data = b"Hello from the filesystem lab!";
    let written = match vfs.write(fd, data) {
        Ok(n) => n,
        Err(e) => {
            println!("  Failed to write: {:?}", e);
            return;
        }
    };
    println!("Step 2: Write {} bytes", written);

    if let Err(e) = vfs.close(fd) {
        println!("  Failed to close: {:?}", e);
        return;
    }
    println!("Step 3: Close fd {}", fd.0);

    let fd2 = match vfs.open("/tmp/lab_test.txt", OpenFlags::READ) {
        Ok(fd) => fd,
        Err(e) => {
            println!("  Failed to re-open: {:?}", e);
            return;
        }
    };
    println!("Step 4: Re-open for reading → fd {}", fd2.0);

    let mut buf = [0u8; 64];
    let n = match vfs.read(fd2, &mut buf) {
        Ok(n) => n,
        Err(e) => {
            println!("  Failed to read: {:?}", e);
            return;
        }
    };
    println!(
        "Step 5: Read {} bytes: \"{}\"",
        n,
        core::str::from_utf8(&buf[..n]).unwrap_or("?")
    );

    if let Err(e) = vfs.close(fd2) {
        println!("  Failed to close: {:?}", e);
        return;
    }

    match vfs.stat("/tmp/lab_test.txt") {
        Ok(stat) => println!(
            "Step 6: Stat → size={}, type={:?}",
            stat.size, stat.inode_type
        ),
        Err(e) => {
            println!("  Failed to stat: {:?}", e);
            return;
        }
    }

    if let Err(e) = vfs.unlink("/tmp/lab_test.txt") {
        println!("  Failed to delete: {:?}", e);
        return;
    }
    println!("Step 7: Delete /tmp/lab_test.txt");

    match vfs.open("/tmp/lab_test.txt", OpenFlags::READ) {
        Err(_) => println!("Step 8: Verified — file no longer exists ✓"),
        Ok(_) => println!("Step 8: ERROR — file still exists!"),
    }

    println!();
    println!("The complete lifecycle: create → write → close → open → read → stat → delete");
    println!();
    println!("Thinking question: What is a file descriptor?");
    println!("  → A small integer (index) into a per-process table of open files.");
    println!("  → It maps to an inode + current read/write offset.");
    println!("  → Closing releases the table entry for reuse.");
    println!();
    println!("✅ Lab complete!");
}

fn lab_allocator_compare() {
    println!("=== Lab: Allocator Algorithm Comparison ===");
    println!();

    let bitmap_start = minios_hal::cpu::read_tsc();
    let mem = minios_memory::get_stats();
    let bitmap_end = minios_hal::cpu::read_tsc();

    let buddy_start = minios_hal::cpu::read_tsc();
    let mut buddy = minios_memory::buddy::BuddyAllocator::new(1024);
    for _ in 0..100 {
        buddy.allocate(0);
    }
    let buddy_end = minios_hal::cpu::read_tsc();

    println!("Bitmap Allocator (real):");
    println!(
        "  Total: {} frames, Free: {}",
        mem.total_frames, mem.free_frames
    );
    println!("  Stats read: {} cycles", bitmap_end - bitmap_start);
    println!();
    println!("Buddy Allocator (simulated, 1024 frames):");
    println!(
        "  100 single-frame allocs: {} cycles",
        buddy_end - buddy_start
    );
    println!(
        "  Allocated: {}, Free: {}",
        buddy.allocated_frames(),
        buddy.free_frames()
    );
    println!();
    println!("Comparison:");
    println!("  {:16} {:>12} {:>12}", "", "Bitmap", "Buddy");
    println!(
        "  {:16} {:>12} {:>12}",
        "Alloc speed", "O(n) scan", "O(log n) split"
    );
    println!(
        "  {:16} {:>12} {:>12}",
        "Free speed", "O(1)", "O(log n) merge"
    );
    println!(
        "  {:16} {:>12} {:>12}",
        "Fragmentation", "External", "Internal (2^n)"
    );
    println!(
        "  {:16} {:>12} {:>12}",
        "Memory overhead", "1 bit/frame", "Free lists"
    );
    println!();
    println!("Linux uses Buddy for page frames because O(log n) allocation");
    println!("is critical when thousands of pages are allocated per second.");
    println!("MiniOS uses Bitmap because it's simpler and our frame count is small.");
    println!();
    println!("✅ Lab complete!");
}

fn lab_scheduler_compare() {
    use minios_common::id::Pid;
    use minios_common::types::Priority;

    println!("=== Lab: Scheduler Algorithm Comparison ===");
    println!();

    let mut mlfq = minios_scheduler::mlfq::MlfqScheduler::new();
    mlfq.add_task(Pid(10), Priority::HIGH);
    mlfq.add_task(Pid(11), Priority::LOW);
    mlfq.set_running(Pid(10), 0);
    for _ in 0..50 {
        mlfq.tick();
    }
    let mlfq_stats = mlfq.stats();

    let mut rr = minios_scheduler::round_robin::RoundRobinScheduler::new(5);
    rr.add_task(Pid(10));
    rr.add_task(Pid(11));
    for _ in 0..50 {
        rr.tick();
    }
    let rr_stats = rr.stats();

    println!("After 50 ticks with 2 tasks:");
    println!("  {:16} {:>10} {:>10}", "", "MLFQ", "Round-Robin");
    println!(
        "  {:16} {:>10} {:>10}",
        "Switches", mlfq_stats.total_switches, rr_stats.total_switches
    );
    println!(
        "  {:16} {:>10} {:>10}",
        "Total ticks", mlfq_stats.total_ticks, rr_stats.total_ticks
    );
    println!();
    println!("Observation: MLFQ switches more because high-priority tasks");
    println!("have shorter time slices. Round-Robin treats all tasks equally.");
    println!();
    println!("Trade-off:");
    println!("  MLFQ   — responsive for interactive tasks, complex rules");
    println!("  RR     — simple and fair, but no priority differentiation");
    println!();
    println!("✅ Lab complete!");
}
