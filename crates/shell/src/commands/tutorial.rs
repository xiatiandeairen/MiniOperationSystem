//! Interactive tutorial guiding new users through MiniOS.

use minios_hal::println;

pub fn cmd_tutorial(_args: &[&str]) {
    println!();
    println!("========================================");
    println!("  MiniOS Interactive Tutorial");
    println!("========================================");
    println!();
    println!("MiniOS is a teaching operating system. Try these commands");
    println!("to explore how an OS works from the inside:");
    println!();
    println!("  Step 1: help              → see all available commands");
    println!("  Step 2: ps                → see running processes");
    println!("  Step 3: meminfo           → see memory usage");
    println!("  Step 4: ls /              → browse the filesystem");
    println!("  Step 5: cat /proc/uptime  → read a virtual file");
    println!("  Step 6: trace follow ls / → trace a command's internals");
    println!("  Step 7: explain ls        → learn how ls works");
    println!("  Step 8: spawn worker      → create a new process");
    println!("  Step 9: sched             → see the scheduler state");
    println!("  Step 10: frames           → visualize memory allocation");
    println!();
    println!("Tip: 'explain <cmd>' explains any command without running it.");
    println!();
}
