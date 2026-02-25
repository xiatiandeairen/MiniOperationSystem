//! Basic shell commands: help, echo, clear, uptime, meminfo.

use minios_hal::{println, serial_println};

/// Lists all available commands with descriptions.
pub fn cmd_help(_args: &[&str]) {
    println!("Available commands:");
    for cmd in super::list_commands() {
        println!("  {:10} - {}", cmd.name, cmd.description);
    }
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

/// Clears the VGA screen.
pub fn cmd_clear(_args: &[&str]) {
    minios_hal::vga::clear_screen();
}

/// Shows the number of timer ticks since boot.
pub fn cmd_uptime(_args: &[&str]) {
    let ticks = minios_hal::interrupts::tick_count();
    println!("Uptime: {} ticks", ticks);
    serial_println!("Uptime: {} ticks", ticks);
}

/// Displays memory statistics (frame allocator + heap).
pub fn cmd_meminfo(_args: &[&str]) {
    let mem = minios_memory::MEMORY_STATS.lock();
    if let Some(ref stats) = *mem {
        println!(
            "Frames: {} free / {} total",
            stats.free_frames, stats.total_frames
        );
        println!(
            "Heap:   {} used / {} free",
            stats.heap_used, stats.heap_free
        );
    } else {
        println!("Memory stats not available");
    }
}
