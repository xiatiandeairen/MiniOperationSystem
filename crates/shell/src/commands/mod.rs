//! Command registry for the shell.

pub mod basic;
pub mod explain;
pub mod fs_cmds;
pub mod mem_cmds;
pub mod proc_cmds;
pub mod sched_cmds;
pub mod trace_cmds;

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
        description: "Trace subsystem (list|tree|stats|clear|export|follow)",
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
        description: "Explain how a command works inside the OS",
        handler: explain::cmd_explain,
    },
    Command {
        name: "tutorial",
        description: "Interactive guide for first-time users",
        handler: explain::cmd_tutorial,
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
