//! Debugging and development commands: log, debug, syscall_demo, trap.

use minios_hal::println;

/// Controls the kernel log system.
pub fn cmd_log(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: log <level|module|history|off> [value]");
        println!("  log level <error|warn|info|debug|trace>");
        println!("  log module <name|all>");
        println!("  log history [count]");
        println!("  log off");
        println!(
            "Current: level={}, module=all",
            minios_hal::log::current_level().as_str()
        );
        return;
    }
    match args[0] {
        "level" => {
            if args.len() < 2 {
                println!("Current: {}", minios_hal::log::current_level().as_str());
                return;
            }
            match minios_hal::log::LogLevel::from_str(args[1]) {
                Some(l) => {
                    minios_hal::log::set_level(l);
                    println!("Log level: {}", l.as_str());
                }
                None => println!("Unknown level. Use: error, warn, info, debug, trace"),
            }
        }
        "module" => {
            if args.len() < 2 {
                println!("Usage: log module <name|all>");
                return;
            }
            minios_hal::log::set_module_filter(args[1]);
            println!("Log module filter: {}", args[1]);
        }
        "history" => {
            let count = if args.len() > 1 {
                args[1]
                    .bytes()
                    .fold(0usize, |acc, b| {
                        if b.is_ascii_digit() {
                            acc * 10 + (b - b'0') as usize
                        } else {
                            acc
                        }
                    })
                    .max(1)
            } else {
                20
            };
            let entries = minios_hal::log::recent_logs(count);
            for e in &entries {
                println!(
                    "[{}] [{}] {}",
                    e.level.as_str(),
                    e.module_str(),
                    e.message_str()
                );
            }
            if entries.is_empty() {
                println!("(no log entries)");
            }
        }
        "off" => {
            minios_hal::log::set_level(minios_hal::log::LogLevel::Error);
            println!("Logging minimized (errors only).");
        }
        _ => println!("Unknown log subcommand. Try: level, module, history, off"),
    }
}

/// Toggles debug mode (trace-level logging for all modules).
pub fn cmd_debug(args: &[&str]) {
    if args.is_empty() || args[0] == "status" {
        println!(
            "Debug mode: log level = {}",
            minios_hal::log::current_level().as_str()
        );
        return;
    }
    match args[0] {
        "on" => {
            minios_hal::log::set_level(minios_hal::log::LogLevel::Trace);
            minios_hal::log::set_module_filter("all");
            println!("Debug mode ON — all logs visible (level=TRACE, module=all)");
        }
        "off" => {
            minios_hal::log::set_level(minios_hal::log::LogLevel::Info);
            minios_hal::log::set_module_filter("all");
            println!("Debug mode OFF — normal logging (level=INFO)");
        }
        _ => println!("Usage: debug <on|off|status>"),
    }
}

/// Demonstrates the int 0x80 syscall mechanism conceptually.
pub fn cmd_syscall_demo(_args: &[&str]) {
    println!("=== System Call Mechanism Demo ===");
    println!();
    println!("How MiniOS syscalls work (current: function call):");
    println!("  1. Shell calls minios_syscall::dispatch(num, arg1, arg2, arg3)");
    println!("  2. Dispatcher matches syscall number \u{2192} calls handler");
    println!("  3. Handler returns result to Shell");
    println!();
    println!("How Linux syscalls work (via int 0x80 / syscall instruction):");
    println!("  1. User puts syscall number in RAX, args in RDI/RSI/RDX");
    println!("  2. 'syscall' instruction traps to kernel (Ring 3 \u{2192} Ring 0)");
    println!("  3. IDT vector 0x80 handler reads registers");
    println!("  4. Dispatcher runs, result goes in RAX");
    println!("  5. 'sysret' returns to user space (Ring 0 \u{2192} Ring 3)");
    println!();
    println!("Key difference: privilege level transition.");
    println!("  MiniOS: everything runs in Ring 0 (kernel mode).");
    println!("  Linux:  user code runs in Ring 3, traps to Ring 0 for syscalls.");
    println!("  This isolation prevents user code from corrupting the kernel.");
    println!();

    let pid = minios_syscall::dispatch(minios_syscall::SYS_GETPID, 0, 0, 0);
    println!("Live demo: sys_getpid() returned PID {}", pid);

    let uptime = minios_syscall::dispatch(minios_syscall::SYS_UPTIME, 0, 0, 0);
    println!("Live demo: sys_uptime() returned {} ticks", uptime);
}

/// Triggers int 0x80 to demonstrate real CPU interrupt dispatch via the IDT.
pub fn cmd_trap(_args: &[&str]) {
    println!("Triggering int 0x80 (software interrupt)...");
    let before = minios_hal::interrupts::syscall_trap_count();

    // SAFETY: int 0x80 is registered in the IDT; this is a controlled software interrupt.
    unsafe {
        core::arch::asm!("int 0x80");
    }

    let after = minios_hal::interrupts::syscall_trap_count();
    println!(
        "int 0x80 handled! Trap count: {} \u{2192} {}",
        before, after
    );
    println!();
    println!("This proves the IDT entry for vector 0x80 is registered");
    println!("and the CPU correctly dispatches to our handler.");
    println!("A full implementation would read RAX for the syscall number.");
}
