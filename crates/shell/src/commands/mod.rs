//! Command registry for the shell.

pub mod basic;
pub mod fs_cmds;
pub mod proc_cmds;
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
        name: "trace",
        description: "Trace subsystem (list|stats|clear|export)",
        handler: trace_cmds::cmd_trace,
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
