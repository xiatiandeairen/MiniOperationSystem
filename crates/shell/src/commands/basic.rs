//! Basic shell commands: help, echo, clear, cheatsheet, faq, feedback,
//! history, scripting utilities, and learning aids.

extern crate alloc;

use minios_hal::println;

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

/// Runs a command once for each item in a list.
pub fn cmd_each(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: each <command> <args...>");
        return;
    }
    let cmd = args[0];
    for item in &args[1..] {
        println!("> {} {}", cmd, item);
        let line = alloc::format!("{} {}", cmd, item);
        let parsed = crate::parser::parse(&line);
        if !parsed.is_empty() {
            if let Some(command) = crate::commands::find_command(parsed.command()) {
                (command.handler)(parsed.args());
            }
        }
    }
}

/// Repeats a command N times.
pub fn cmd_repeat(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: repeat <N> <command> [args...]");
        return;
    }
    let n = parse_usize_basic(args[0]).unwrap_or(1).min(100);
    let cmd_line: alloc::string::String = args[1..].join(" ");
    for i in 0..n {
        println!("--- iteration {} ---", i + 1);
        let parsed = crate::parser::parse(&cmd_line);
        if !parsed.is_empty() {
            if let Some(command) = crate::commands::find_command(parsed.command()) {
                (command.handler)(parsed.args());
            }
        }
    }
}

fn parse_usize_basic(s: &str) -> Option<usize> {
    let mut r: usize = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        r = r.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(r)
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

/// Exports a summary of the current learning session.
pub fn cmd_export_session(_args: &[&str]) {
    println!("=== Session Export ===");
    let hist_count = crate::shell::HISTORY.lock().len();
    println!("Commands executed: {}", hist_count);
    for i in 0..hist_count {
        if let Some(cmd) = crate::shell::HISTORY.lock().get(i) {
            println!("  {}. {}", i + 1, cmd);
        }
    }
    println!();
    let completed = crate::commands::journey::completed_count();
    println!("Journey progress: {}/17 steps", completed);
    println!("---");
}

/// Prints a quick reference card of all MiniOS command categories.
pub fn cmd_cheatsheet(_args: &[&str]) {
    println!("+---------------------------------------+");
    println!("|     MiniOS Quick Reference Card       |");
    println!("+---------------------------------------+");
    println!("| LEARN    tutorial explain compare lab |");
    println!("| PROCESS  ps spawn kill signal sched   |");
    println!("| MEMORY   meminfo frames pagetable     |");
    println!("| FILES    ls cat mkdir write touch rm   |");
    println!("| TRACE    trace follow/tree/filter/log  |");
    println!("| TEST     crash lab bench               |");
    println!("| SCRIPT   run each repeat alias history |");
    println!("| TEXT     head grep wc                   |");
    println!("| STATUS   top uptime interrupts version |");
    println!("| HELP     help man explain cheatsheet   |");
    println!("+---------------------------------------+");
}

/// Answers common learner questions about MiniOS.
pub fn cmd_faq(_args: &[&str]) {
    println!("=== Frequently Asked Questions ===");
    println!();
    println!("Q: How do I start learning?");
    println!("A: Type 'tutorial' for a guided 10-step walkthrough.");
    println!();
    println!("Q: What does 'explain' do?");
    println!("A: Shows how a command works internally without running it.");
    println!();
    println!("Q: Can I break the system?");
    println!("A: Yes! 'crash oom' safely demonstrates failures. Try it!");
    println!();
    println!("Q: How do I see what the OS is doing internally?");
    println!("A: 'trace follow <cmd>' shows every system call in a command.");
    println!("   'log level debug' enables detailed kernel logging.");
    println!();
    println!("Q: How do I track my progress?");
    println!("A: 'journey' shows your learning path. 'graduation' for final report.");
    println!();
    println!("Q: Is this a real operating system?");
    println!("A: Yes! It boots on real x86-64 hardware (via QEMU). It has real");
    println!("   memory management, process scheduling, and a filesystem.");
}

/// Prints a feedback prompt directing users to the project's issue tracker.
pub fn cmd_feedback(_args: &[&str]) {
    println!("=== Help Us Improve MiniOS ===");
    println!();
    println!("Thank you for using MiniOS! Your feedback matters.");
    println!();
    println!("Please tell us:");
    println!("  1. What was the most valuable thing you learned?");
    println!("  2. What was confusing or unclear?");
    println!("  3. What feature would you add?");
    println!("  4. Would you recommend MiniOS? (1-5)");
    println!();
    println!("Share feedback via GitHub Issues:");
    println!("  https://github.com/xiatiandeairen/MiniOperationSystem/issues");
    println!();
    println!("Your journey progress:");
    let done = crate::commands::journey::completed_count();
    println!("  {}/17 learning steps completed", done);
    let cmds = crate::shell::HISTORY.lock().len();
    println!("  {} commands executed this session", cmds);
}

/// Prints a structured course outline for using MiniOS as a teaching tool.
pub fn cmd_syllabus(_args: &[&str]) {
    println!("=== MiniOS Operating Systems Syllabus ===");
    println!();
    println!("Module 1: Process Management (2 hours)");
    println!("  Concepts: PCB, state machine, scheduling algorithms");
    println!("  Commands: ps, spawn, kill, signal, sched, compare scheduler");
    println!("  Lab: lab scheduler-fairness");
    println!("  Reading: explain ps, explain spawn, explain sched");
    println!();
    println!("Module 2: Memory Management (2 hours)");
    println!("  Concepts: Physical frames, virtual pages, heap allocation");
    println!("  Commands: meminfo, frames, pagetable, alloc");
    println!("  Lab: lab memory-usage, lab page-table-walk");
    println!("  Reading: explain meminfo, explain frames, explain pagetable");
    println!();
    println!("Module 3: File Systems (1.5 hours)");
    println!("  Concepts: VFS, inodes, file descriptors, directory tree");
    println!("  Commands: ls, cat, mkdir, write, touch, rm");
    println!("  Lab: lab fs-operations");
    println!("  Reading: explain ls, explain cat, compare filesystem");
    println!();
    println!("Module 4: System Calls & IPC (1.5 hours)");
    println!("  Concepts: Syscall interface, message passing, tracing");
    println!("  Commands: trace follow, trace tree, log, compare syscall/ipc");
    println!("  Lab: lab trace-overhead");
    println!("  Reading: explain trace, compare syscall, compare ipc");
    println!();
    println!("Module 5: Fault Handling (1 hour)");
    println!("  Concepts: Interrupts, exceptions, OOM, stack overflow");
    println!("  Commands: crash oom, crash stack, crash divide-zero, interrupts");
    println!("  Reading: explain log");
    println!();
    println!("Total: ~8 hours of guided hands-on learning");
}
