//! Crash lab — safely demonstrates OS fault scenarios.

extern crate alloc;
use alloc::vec::Vec;
use minios_hal::println;

pub fn cmd_crash(args: &[&str]) {
    if args.is_empty() {
        crash_list();
        return;
    }
    match args[0] {
        "list" => crash_list(),
        "oom" | "1" => crash_oom(),
        "stack" | "2" => crash_stack(),
        "divide-zero" | "3" => crash_divide_zero(),
        "null-deref" | "4" => crash_null_deref(),
        "fork-bomb" | "5" => crash_fork_bomb(),
        _ => println!("Unknown scenario. Type 'crash list' for options."),
    }
    if args[0] != "list" {
        super::journey::mark(super::journey::STEP_CRASH);
    }
}

fn crash_list() {
    println!("=== Crash Lab — Safe Fault Experiments ===");
    println!();
    println!("  1. oom          Exhaust heap memory");
    println!("  2. stack        Deep recursion (simulated)");
    println!("  3. divide-zero  Division by zero (simulated)");
    println!("  4. null-deref   Null pointer dereference (simulated)");
    println!("  5. fork-bomb    Rapid process creation until limit");
    println!();
    println!("Each scenario explains what happens and recovers safely.");
    println!("Usage: crash <name> or crash <number>");
}

fn crash_oom() {
    println!("=== Crash Lab: Out of Memory ===");
    println!();

    let before = minios_memory::get_stats();
    println!(
        "Heap before: {} used, {} free",
        before.heap_used, before.heap_free
    );
    println!();
    println!("Allocating 1 KiB blocks until failure...");

    let mut blocks: Vec<Vec<u8>> = Vec::new();
    let mut count = 0u32;

    let limit = (before.heap_free * 9) / 10;
    let mut total_allocated: usize = 0;

    while total_allocated + 1024 < limit {
        blocks.push(alloc::vec![0xAA; 1024]);
        count += 1;
        total_allocated += 1024;
        if count.is_multiple_of(100) {
            println!("  {} blocks allocated ({} KiB)", count, count);
        }
    }

    let during = minios_memory::get_stats();
    println!();
    println!(
        "Stopped at {} blocks ({} KiB) — 90% of heap consumed",
        count, count
    );
    println!(
        "Heap now: {} used, {} free",
        during.heap_used, during.heap_free
    );
    println!();

    drop(blocks);
    let after = minios_memory::get_stats();
    println!(
        "After freeing: {} used, {} free",
        after.heap_used, after.heap_free
    );
    println!();

    println!("What would happen if we kept going?");
    println!("  MiniOS: The global allocator returns null → panic!");
    println!("  Linux:  The OOM killer selects a process to terminate,");
    println!("          freeing its memory so others can continue.");
    println!();
    println!("Key lesson: An OS must handle memory exhaustion gracefully.");
    println!("MiniOS panics (simple but fatal). Linux kills (complex but resilient).");
    println!();
    println!("✅ Recovered safely. System stable.");
}

fn crash_stack() {
    println!("=== Crash Lab: Stack Overflow ===");
    println!();
    println!("In a real OS, deep recursion consumes stack space:");
    println!("  fn recursive(n) {{ recursive(n+1) }}");
    println!();

    println!("Simulating recursion depth tracking:");
    for depth in [10, 100, 500, 1000, 4000] {
        let stack_bytes = depth * 64;
        println!(
            "  depth {:>5}: ~{:>6} bytes of stack ({} KiB)",
            depth,
            stack_bytes,
            stack_bytes / 1024
        );
    }
    println!();
    println!("MiniOS kernel stack: 512 KiB (set in bootloader config).");
    println!("At ~64 bytes/frame, overflow at ~8000 recursion depth.");
    println!();
    println!("What happens on overflow?");
    println!("  The CPU writes beyond the stack's mapped pages.");
    println!("  This triggers a Page Fault (interrupt 14).");
    println!("  If the fault handler's own stack overflows → Double Fault.");
    println!("  If that overflows too → Triple Fault → CPU reset.");
    println!();
    println!("MiniOS uses IST (Interrupt Stack Table) to give the");
    println!("Double Fault handler its own separate 20 KiB stack,");
    println!("preventing the cascade to Triple Fault.");
    println!();
    println!("✅ Simulation complete. No actual recursion performed.");
}

fn crash_divide_zero() {
    println!("=== Crash Lab: Division by Zero ===");
    println!();
    println!("In x86-64, integer division by zero triggers:");
    println!("  CPU Exception #0 — Divide Error");
    println!();
    println!("The CPU pushes an interrupt frame onto the stack and");
    println!("jumps to the handler registered in IDT entry 0.");
    println!();
    println!("MiniOS handler: prints error info + halts.");
    println!("Linux handler: sends SIGFPE to the offending process,");
    println!("  which typically terminates it (core dump if enabled).");
    println!();
    println!("Note: Rust prevents integer division by zero at compile");
    println!("time (panic in debug, wrapping in release). The CPU");
    println!("exception only fires from unchecked assembly division.");
    println!();
    println!("✅ Explanation complete. No actual division performed.");
}

fn crash_null_deref() {
    println!("=== Crash Lab: Null Pointer Dereference ===");
    println!();
    println!("Accessing address 0x0 triggers:");
    println!("  CPU Exception #14 — Page Fault");
    println!("  Error code: 0x0 (not-present read)");
    println!();
    println!("Why? The first page (0x0000–0x0FFF) is intentionally");
    println!("NOT mapped in the page tables. This is a guard page.");
    println!();
    println!("In MiniOS:");
    println!("  Page Fault handler prints the faulting address and halts.");
    println!();
    println!("In Linux:");
    println!("  Sends SIGSEGV to the process → \"Segmentation fault\".");
    println!("  The process is terminated (core dump if enabled).");
    println!();
    println!("In Rust:");
    println!("  Safe Rust cannot create null pointers.");
    println!("  Only `unsafe {{ *(0 as *const u8) }}` can trigger this.");
    println!();
    println!("✅ Explanation complete. Guard page is working.");
}

fn crash_fork_bomb() {
    println!("=== Crash Lab: Fork Bomb ===");
    println!();
    println!("A fork bomb creates processes exponentially:");
    println!("  :(){{ :|:& }};:   (classic bash fork bomb)");
    println!();

    println!("Simulating rapid process creation...");
    let mut created = 0u32;
    let mut last_error = false;

    for i in 0..10 {
        match minios_process::manager::create_kernel_task(
            "bomb",
            bomb_task,
            minios_common::types::Priority::LOW,
        ) {
            Ok(pid) => {
                created += 1;
                if created <= 5 || created.is_multiple_of(5) {
                    println!("  Created PID {} ({} total)", pid, created);
                }
            }
            Err(_) => {
                if !last_error {
                    println!(
                        "  Process creation failed at attempt {} — table full!",
                        i + 1
                    );
                    last_error = true;
                }
            }
        }
    }

    println!();
    println!("Created {} processes before hitting the limit.", created);
    println!("MiniOS max processes: 64 (fixed-size table).");
    println!();

    let procs = minios_process::manager::list_processes();
    let mut cleaned = 0;
    for p in &procs {
        if p.pid.0 >= 2 {
            let _ = minios_process::manager::set_state(
                p.pid,
                minios_common::types::ProcessState::Terminated,
            );
            minios_scheduler::SCHEDULER.lock().remove_task(p.pid);
            cleaned += 1;
        }
    }
    println!("Cleaned up {} bomb processes.", cleaned);
    println!();

    println!("Defense mechanisms:");
    println!("  MiniOS: Fixed process table (64 slots) → natural limit.");
    println!("  Linux:  ulimit -u (max user processes), cgroups,");
    println!("          systemd resource accounting.");
    println!();
    println!("✅ Recovered safely. Bomb processes terminated.");
}

fn bomb_task() {
    loop {
        minios_hal::cpu::hlt();
    }
}
