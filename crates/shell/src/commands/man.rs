//! The `man` command — quick usage reference for shell commands.

use minios_hal::println;

pub fn cmd_man(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: man <command>");
        println!("Shows detailed usage information for a command.");
        return;
    }
    match args[0] {
        "ls" => {
            println!("ls [path]");
            println!("  Lists directory contents. Default: /");
        }
        "cat" => {
            println!("cat <file>");
            println!("  Displays file contents.");
        }
        "trace" => {
            println!("trace <list|tree|follow|filter|stats|clear|export>");
            println!("  Manage the kernel trace system.");
        }
        "log" => {
            println!("log <level|module|history|off> [value]");
            println!("  Control kernel logging.");
        }
        "spawn" => {
            println!("spawn [name]");
            println!("  Create a background kernel task.");
        }
        "kill" => {
            println!("kill <pid>");
            println!("  Terminate a process by PID.");
        }
        "crash" => {
            println!("crash <oom|stack|divide-zero|null-deref|fork-bomb>");
            println!("  Safely demonstrate fault scenarios.");
        }
        "compare" => {
            println!("compare <scheduler|memory|filesystem|ipc|syscall>");
            println!("  Compare MiniOS vs Linux design.");
        }
        "lab" => {
            println!("lab <1-5|name>");
            println!("  Run interactive OS experiments.");
        }
        "bench" => {
            println!("bench <alloc|trace|fs>");
            println!("  Run performance benchmarks.");
        }
        _ => println!(
            "No manual entry for '{}'. Try 'explain {}' for how it works.",
            args[0], args[0]
        ),
    }
}
