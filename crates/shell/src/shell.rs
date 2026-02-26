//! Shell main loop: prompt, read, parse, dispatch.

extern crate alloc;

use crate::commands;
use crate::input::LineBuffer;
use crate::parser;
use alloc::vec::Vec;
use minios_hal::{print, println, serial_println};
use spin::Mutex;

// ---------------------------------------------------------------------------
// Command history (ring buffer of 32 entries, each up to 256 bytes)
// ---------------------------------------------------------------------------

const MAX_HISTORY: usize = 32;
const MAX_ENTRY: usize = 256;

pub(crate) struct History {
    entries: [[u8; MAX_ENTRY]; MAX_HISTORY],
    lengths: [usize; MAX_HISTORY],
    count: usize,
    next: usize,
}

impl History {
    const fn new() -> Self {
        Self {
            entries: [[0; MAX_ENTRY]; MAX_HISTORY],
            lengths: [0; MAX_HISTORY],
            count: 0,
            next: 0,
        }
    }

    pub(crate) fn push(&mut self, line: &str) {
        let len = line.len().min(MAX_ENTRY);
        self.entries[self.next][..len].copy_from_slice(&line.as_bytes()[..len]);
        self.lengths[self.next] = len;
        self.next = (self.next + 1) % MAX_HISTORY;
        if self.count < MAX_HISTORY {
            self.count += 1;
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&str> {
        if index >= self.count {
            return None;
        }
        let actual = if self.count < MAX_HISTORY {
            index
        } else {
            (self.next + index) % MAX_HISTORY
        };
        let len = self.lengths[actual];
        core::str::from_utf8(&self.entries[actual][..len]).ok()
    }

    pub(crate) fn len(&self) -> usize {
        self.count.min(MAX_HISTORY)
    }

    #[allow(dead_code)]
    pub(crate) fn last(&self) -> Option<&str> {
        if self.count == 0 {
            return None;
        }
        let idx = if self.next == 0 {
            MAX_HISTORY - 1
        } else {
            self.next - 1
        };
        let len = self.lengths[idx];
        core::str::from_utf8(&self.entries[idx][..len]).ok()
    }
}

pub static HISTORY: Mutex<History> = Mutex::new(History::new());

/// Tries the PS/2 keyboard first, then falls back to the serial port.
fn read_char() -> Option<u8> {
    minios_hal::keyboard::read_key().or_else(minios_hal::serial::read_byte)
}

/// Reads one line of input from the keyboard or serial port, echoing to VGA.
///
/// Polls both input sources with `hlt` between polls to avoid
/// busy-waiting. Handles backspace and returns on Enter.
fn read_line(buf: &mut LineBuffer) {
    buf.clear();
    loop {
        if let Some(ch) = read_char() {
            match ch {
                b'\n' | 13 => {
                    println!();
                    return;
                }
                8 | 127 if !buf.is_empty() => {
                    buf.backspace();
                    print!("\x08 \x08");
                }
                b'\t' => {
                    let prefix = buf.as_str();
                    if !prefix.is_empty() {
                        if let Some(match_name) = find_completion(prefix) {
                            while !buf.is_empty() {
                                buf.backspace();
                                print!("\x08 \x08");
                            }
                            for b in match_name.bytes() {
                                buf.push(b);
                                print!("{}", b as char);
                            }
                        }
                    }
                }
                ch if ch >= 0x20 => {
                    buf.push(ch);
                    print!("{}", ch as char);
                }
                _ => {}
            }
        } else {
            minios_hal::cpu::hlt();
        }
    }
}

/// Prints the shell prompt in green.
fn print_prompt() {
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::GREEN);
    print!("MiniOS $ ");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
}

/// Returns a unique command name matching the given prefix, or `None` if
/// zero or multiple commands match (ambiguous).
fn find_completion(prefix: &str) -> Option<&'static str> {
    let mut found: Option<&str> = None;
    for cmd in crate::commands::list_commands() {
        if cmd.name.starts_with(prefix) {
            if found.is_some() {
                return None;
            }
            found = Some(cmd.name);
        }
    }
    found
}

/// Returns a command whose name shares the first two characters with
/// `input`, or `None` if no plausible match exists.
fn find_similar(input: &str) -> Option<&'static str> {
    if input.len() < 2 {
        return None;
    }
    let prefix = &input[..2];
    for cmd in crate::commands::list_commands() {
        if cmd.name.starts_with(prefix) && cmd.name != input {
            return Some(cmd.name);
        }
    }
    None
}

/// Runs the interactive shell loop. This function never returns.
pub fn run_shell() -> ! {
    crate::commands::env_cmds::init_defaults();

    println!("MiniOS Shell v0.7");
    println!("Type 'tutorial' to start learning, or 'help' for all commands.\n");
    serial_println!("Shell started");

    super::commands::basic::cmd_run(&["/etc/init.sh"]);

    let mut buf = LineBuffer::new();
    let mut last_cmd = [0u8; MAX_ENTRY];
    let mut last_cmd_len: usize = 0;

    loop {
        print_prompt();
        read_line(&mut buf);

        let line = buf.as_str();
        if line.is_empty() {
            continue;
        }

        // Handle !! (repeat last command)
        if line == "!!" {
            if last_cmd_len == 0 {
                println!("No previous command.");
                continue;
            }
            let recalled = core::str::from_utf8(&last_cmd[..last_cmd_len]).unwrap_or("");
            println!("{}", recalled);
            buf.clear();
            for b in recalled.bytes() {
                buf.push(b);
            }
        }

        let line = buf.as_str();

        // Save to local last-command buffer and global history
        let len = line.len().min(MAX_ENTRY);
        last_cmd[..len].copy_from_slice(&line.as_bytes()[..len]);
        last_cmd_len = len;
        HISTORY.lock().push(line);

        serial_println!("shell> {}", line);

        // Pipe operator: capture left-side output and feed to right-side
        if line.contains(" | ") {
            execute_pipe(line);
            continue;
        }

        dispatch_line(line);
    }
}

/// Dispatches a single command line (with alias resolution).
fn dispatch_line(line: &str) {
    let parsed = parser::parse(line);
    if parsed.is_empty() {
        return;
    }

    let cmd_name = parsed.command();
    let args = parsed.args();

    // Check aliases before built-in commands
    let alias_expansion = {
        let aliases = crate::commands::alias::ALIASES.lock();
        aliases.get(cmd_name).map(|s| {
            let mut buf = [0u8; 256];
            let len = s.len().min(256);
            buf[..len].copy_from_slice(&s.as_bytes()[..len]);
            (buf, len)
        })
    };
    if let Some((buf, len)) = alias_expansion {
        if let Ok(expanded) = core::str::from_utf8(&buf[..len]) {
            // Append original args to the alias expansion
            if args.is_empty() {
                dispatch_line(expanded);
            } else {
                let mut full = [0u8; 256];
                let mut pos = 0;
                let exp_bytes = expanded.as_bytes();
                let copy_len = exp_bytes.len().min(256);
                full[..copy_len].copy_from_slice(&exp_bytes[..copy_len]);
                pos += copy_len;
                for arg in args {
                    if pos < 255 {
                        full[pos] = b' ';
                        pos += 1;
                    }
                    let ab = arg.as_bytes();
                    let alen = ab.len().min(255 - pos);
                    full[pos..pos + alen].copy_from_slice(&ab[..alen]);
                    pos += alen;
                }
                if let Ok(full_str) = core::str::from_utf8(&full[..pos]) {
                    dispatch_line(full_str);
                }
            }
            return;
        }
    }

    match commands::find_command(cmd_name) {
        Some(command) => (command.handler)(args),
        None => {
            let suggestion = find_similar(cmd_name);
            minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::RED);
            println!("Unknown command: {}", cmd_name);
            if let Some(s) = suggestion {
                minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
                println!("Did you mean: {}?", s);
            }
            minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
        }
    }
}

/// Executes a command and captures its printed output.
fn capture_command(line: &str) -> Vec<u8> {
    *minios_hal::vga::PIPE_BUFFER.lock() = Some(Vec::new());
    dispatch_line(line);
    minios_hal::vga::PIPE_BUFFER
        .lock()
        .take()
        .unwrap_or_default()
}

/// Executes a pipe: `left | right`.
fn execute_pipe(line: &str) {
    let parts: Vec<&str> = line.splitn(2, " | ").collect();
    if parts.len() != 2 {
        println!("Invalid pipe syntax");
        return;
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    let captured = capture_command(left);
    let captured_str = core::str::from_utf8(&captured).unwrap_or("");

    // Parse the right-side command
    let parsed = parser::parse(right);
    if parsed.is_empty() {
        return;
    }
    let cmd = parsed.command();
    let args = parsed.args();

    match cmd {
        "head" => pipe_head(captured_str, args),
        "grep" => pipe_grep(captured_str, args),
        "wc" => pipe_wc(captured_str),
        _ => {
            // For other commands, print the captured output (pass-through)
            print!("{}", captured_str);
        }
    }
}

fn pipe_head(stdin: &str, args: &[&str]) {
    let n = if args.is_empty() {
        10
    } else {
        parse_usize(args[0]).unwrap_or(10)
    };
    for (i, line) in stdin.lines().enumerate() {
        if i >= n {
            break;
        }
        println!("{}", line);
    }
}

fn pipe_grep(stdin: &str, args: &[&str]) {
    if args.is_empty() {
        println!("grep: missing pattern");
        return;
    }
    let pattern = args[0];
    for line in stdin.lines() {
        if line.contains(pattern) {
            println!("{}", line);
        }
    }
}

fn pipe_wc(stdin: &str) {
    let lines = stdin.lines().count();
    let words = stdin.split_whitespace().count();
    let bytes = stdin.len();
    println!("  {:>6} {:>6} {:>6}", lines, words, bytes);
}

fn parse_usize(s: &str) -> Option<usize> {
    let mut result: usize = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(result)
}
