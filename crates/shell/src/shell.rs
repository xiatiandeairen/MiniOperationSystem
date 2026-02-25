//! Shell main loop: prompt, read, parse, dispatch.

use crate::commands;
use crate::input::LineBuffer;
use crate::parser;
use minios_hal::{print, println, serial_println};

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
                8 | 127 => {
                    if !buf.is_empty() {
                        buf.backspace();
                        print!("\x08 \x08");
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

/// Prints the shell prompt.
fn print_prompt() {
    print!("MiniOS $ ");
}

/// Runs the interactive shell loop. This function never returns.
pub fn run_shell() -> ! {
    println!("MiniOS Shell v0.1");
    println!("Type 'help' for available commands.\n");
    serial_println!("Shell started");

    let mut buf = LineBuffer::new();

    loop {
        print_prompt();
        read_line(&mut buf);

        let line = buf.as_str();
        if line.is_empty() {
            continue;
        }

        serial_println!("shell> {}", line);

        let parsed = parser::parse(line);
        if parsed.is_empty() {
            continue;
        }

        let cmd_name = parsed.command();
        let args = parsed.args();

        match commands::find_command(cmd_name) {
            Some(command) => (command.handler)(args),
            None => println!("Unknown command: {}", cmd_name),
        }
    }
}
