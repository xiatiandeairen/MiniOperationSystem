//! Shell main loop: prompt, read, parse, dispatch.

use crate::commands;
use crate::input::LineBuffer;
use crate::parser;
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

/// Runs the interactive shell loop. This function never returns.
pub fn run_shell() -> ! {
    crate::commands::env_cmds::init_defaults();

    println!("MiniOS Shell v0.4");
    println!("Type 'tutorial' to start learning, or 'help' for all commands.\n");
    serial_println!("Shell started");

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

        let parsed = parser::parse(line);
        if parsed.is_empty() {
            continue;
        }

        let cmd_name = parsed.command();
        let args = parsed.args();

        match commands::find_command(cmd_name) {
            Some(command) => (command.handler)(args),
            None => {
                minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::RED);
                println!("Unknown command: {}", cmd_name);
                minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
            }
        }
    }
}
