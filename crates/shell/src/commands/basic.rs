//! Basic shell commands: help, echo, clear, uptime, meminfo.

use minios_hal::{println, serial_println};

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

/// Shows the number of timer ticks since boot.
pub fn cmd_uptime(_args: &[&str]) {
    let ticks = minios_hal::interrupts::tick_count();
    println!("Uptime: {} ticks", ticks);
    serial_println!("Uptime: {} ticks", ticks);
    super::journey::mark(super::journey::STEP_UPTIME);
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
