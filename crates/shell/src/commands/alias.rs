//! Command aliases — user-defined shortcuts for command strings.

extern crate alloc;

use alloc::string::String;
use minios_hal::println;
use spin::Mutex;

const MAX_ALIASES: usize = 16;
const MAX_NAME: usize = 16;
const MAX_CMD: usize = 128;

struct Alias {
    name: [u8; MAX_NAME],
    name_len: usize,
    cmd: [u8; MAX_CMD],
    cmd_len: usize,
}

pub struct Aliases {
    entries: [Option<Alias>; MAX_ALIASES],
}

impl Aliases {
    const fn new() -> Self {
        const NONE: Option<Alias> = None;
        Self {
            entries: [NONE; MAX_ALIASES],
        }
    }

    pub fn set(&mut self, name: &str, cmd: &str) {
        let nlen = name.len().min(MAX_NAME);
        let clen = cmd.len().min(MAX_CMD);

        // Update existing alias if found
        for entry in self.entries.iter_mut().flatten() {
            if entry.name_len == nlen && entry.name[..nlen] == *name.as_bytes() {
                entry.cmd[..clen].copy_from_slice(&cmd.as_bytes()[..clen]);
                entry.cmd_len = clen;
                return;
            }
        }

        // Insert into first empty slot
        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                let mut alias = Alias {
                    name: [0; MAX_NAME],
                    name_len: 0,
                    cmd: [0; MAX_CMD],
                    cmd_len: 0,
                };
                alias.name[..nlen].copy_from_slice(&name.as_bytes()[..nlen]);
                alias.name_len = nlen;
                alias.cmd[..clen].copy_from_slice(&cmd.as_bytes()[..clen]);
                alias.cmd_len = clen;
                *slot = Some(alias);
                return;
            }
        }
        println!("alias: too many aliases (max {})", MAX_ALIASES);
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        for entry in self.entries.iter().flatten() {
            if entry.name_len == name.len() && entry.name[..entry.name_len] == *name.as_bytes() {
                return core::str::from_utf8(&entry.cmd[..entry.cmd_len]).ok();
            }
        }
        None
    }
}

pub static ALIASES: Mutex<Aliases> = Mutex::new(Aliases::new());

/// `alias` — list all aliases, or `alias <name> <command...>` to create one.
pub fn cmd_alias(args: &[&str]) {
    if args.is_empty() {
        let aliases = ALIASES.lock();
        let mut any = false;
        for entry in aliases.entries.iter().flatten() {
            let name = core::str::from_utf8(&entry.name[..entry.name_len]).unwrap_or("?");
            let cmd = core::str::from_utf8(&entry.cmd[..entry.cmd_len]).unwrap_or("?");
            println!("  {} = '{}'", name, cmd);
            any = true;
        }
        if !any {
            println!("No aliases defined. Usage: alias <name> <command...>");
        }
        return;
    }
    if args.len() < 2 {
        println!("Usage: alias <name> <command...>");
        return;
    }
    let name = args[0];
    // Join remaining args with spaces
    let cmd = join_args(&args[1..]);
    ALIASES.lock().set(name, &cmd);
    println!("Alias set: {} = '{}'", name, cmd);
}

fn join_args(args: &[&str]) -> String {
    let mut result = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        result.push_str(arg);
    }
    result
}
