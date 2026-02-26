//! Command registry for the shell.

pub mod alias;
pub mod basic;
pub mod bench;
pub mod compare;
pub mod crash;
pub mod env_cmds;
pub mod errors;
pub mod explain;
pub mod fs_cmds;
pub mod journey;
pub mod lab;
pub mod man;
pub mod mem_cmds;
pub mod proc_cmds;
pub mod sched_cmds;
pub mod text;
pub mod trace_cmds;
pub mod tutorial;

/// A shell command with its name, description, and handler function.
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: fn(&[&str]),
}

/// Static list of all available commands.
static COMMANDS: &[Command] = &[
    Command {
        name: "help",
        description: "List all available commands",
        handler: basic::cmd_help,
    },
    Command {
        name: "echo",
        description: "Print arguments to the screen",
        handler: basic::cmd_echo,
    },
    Command {
        name: "clear",
        description: "Clear the VGA screen",
        handler: basic::cmd_clear,
    },
    Command {
        name: "uptime",
        description: "Show tick count since boot",
        handler: basic::cmd_uptime,
    },
    Command {
        name: "meminfo",
        description: "Show memory statistics",
        handler: basic::cmd_meminfo,
    },
    Command {
        name: "interrupts",
        description: "Show interrupt statistics",
        handler: basic::cmd_interrupts,
    },
    Command {
        name: "ls",
        description: "List directory contents",
        handler: fs_cmds::cmd_ls,
    },
    Command {
        name: "cat",
        description: "Print file contents",
        handler: fs_cmds::cmd_cat,
    },
    Command {
        name: "mkdir",
        description: "Create a directory",
        handler: fs_cmds::cmd_mkdir,
    },
    Command {
        name: "touch",
        description: "Create an empty file",
        handler: fs_cmds::cmd_touch,
    },
    Command {
        name: "write",
        description: "Write content to a file",
        handler: fs_cmds::cmd_write,
    },
    Command {
        name: "pwd",
        description: "Print working directory",
        handler: fs_cmds::cmd_pwd,
    },
    Command {
        name: "ps",
        description: "List all processes",
        handler: proc_cmds::cmd_ps,
    },
    Command {
        name: "spawn",
        description: "Spawn a background kernel task",
        handler: sched_cmds::cmd_spawn,
    },
    Command {
        name: "kill",
        description: "Terminate a process by PID",
        handler: sched_cmds::cmd_kill,
    },
    Command {
        name: "sched",
        description: "Show scheduler queue stats",
        handler: sched_cmds::cmd_sched,
    },
    Command {
        name: "nice",
        description: "Change process priority",
        handler: sched_cmds::cmd_nice,
    },
    Command {
        name: "trace",
        description: "Trace subsystem (list|tree|stats|clear|export|follow|filter)",
        handler: trace_cmds::cmd_trace,
    },
    Command {
        name: "pagetable",
        description: "Decompose virtual address into page table indices",
        handler: mem_cmds::cmd_pagetable,
    },
    Command {
        name: "frames",
        description: "Show physical frame usage with visual bar",
        handler: mem_cmds::cmd_frames,
    },
    Command {
        name: "alloc",
        description: "Allocate heap memory and show result",
        handler: mem_cmds::cmd_alloc,
    },
    Command {
        name: "explain",
        description: "Explain how a command works internally",
        handler: explain::cmd_explain,
    },
    Command {
        name: "tutorial",
        description: "Interactive guide to exploring MiniOS",
        handler: tutorial::cmd_tutorial,
    },
    Command {
        name: "compare",
        description: "Compare MiniOS vs Linux design choices",
        handler: compare::cmd_compare,
    },
    Command {
        name: "lab",
        description: "Run interactive OS learning experiments",
        handler: lab::cmd_lab,
    },
    Command {
        name: "crash",
        description: "Safe fault experiments (oom|stack|divide-zero|null|fork-bomb)",
        handler: crash::cmd_crash,
    },
    Command {
        name: "run",
        description: "Execute commands from a script file",
        handler: basic::cmd_run,
    },
    Command {
        name: "each",
        description: "Run a command for each item in a list",
        handler: basic::cmd_each,
    },
    Command {
        name: "repeat",
        description: "Repeat a command N times",
        handler: basic::cmd_repeat,
    },
    Command {
        name: "syllabus",
        description: "Show structured OS course outline",
        handler: basic::cmd_syllabus,
    },
    Command {
        name: "history",
        description: "Show command history",
        handler: basic::cmd_history,
    },
    Command {
        name: "signal",
        description: "Send signal to process (stop|continue|kill)",
        handler: sched_cmds::cmd_signal,
    },
    Command {
        name: "set",
        description: "Set an environment variable",
        handler: env_cmds::cmd_set,
    },
    Command {
        name: "env",
        description: "List all environment variables",
        handler: env_cmds::cmd_env,
    },
    Command {
        name: "head",
        description: "Show first N lines of a file",
        handler: text::cmd_head,
    },
    Command {
        name: "grep",
        description: "Search for a pattern in a file",
        handler: text::cmd_grep,
    },
    Command {
        name: "wc",
        description: "Count lines, words, bytes in a file",
        handler: text::cmd_wc,
    },
    Command {
        name: "alias",
        description: "Create or list command aliases",
        handler: alias::cmd_alias,
    },
    Command {
        name: "log",
        description: "Control kernel log system (level, module, history)",
        handler: basic::cmd_log,
    },
    Command {
        name: "debug",
        description: "Toggle debug mode (on|off|status)",
        handler: basic::cmd_debug,
    },
    Command {
        name: "journey",
        description: "Show your MiniOS learning journey progress",
        handler: journey::cmd_journey,
    },
    Command {
        name: "graduation",
        description: "Show learning completion report",
        handler: journey::cmd_graduation,
    },
    Command {
        name: "top",
        description: "Show system-wide status snapshot (processes, memory, IRQs)",
        handler: proc_cmds::cmd_top,
    },
    Command {
        name: "bench",
        description: "Run built-in performance benchmarks (alloc, trace, fs)",
        handler: bench::cmd_bench,
    },
    Command {
        name: "memmap",
        description: "Show ASCII memory layout diagram",
        handler: mem_cmds::cmd_memmap,
    },
    Command {
        name: "pstree",
        description: "Show process hierarchy tree",
        handler: proc_cmds::cmd_pstree,
    },
    Command {
        name: "safety",
        description: "Show unsafe code audit summary",
        handler: basic::cmd_safety,
    },
    Command {
        name: "report",
        description: "Export structured learning progress report",
        handler: journey::cmd_report,
    },
    Command {
        name: "man",
        description: "Quick usage reference for a command",
        handler: man::cmd_man,
    },
];

/// Finds a command by name.
pub fn find_command(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

/// Returns the full list of registered commands.
pub fn list_commands() -> &'static [Command] {
    COMMANDS
}
