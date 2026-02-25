//! VGA text-mode display driver.
//!
//! Provides a global [`WRITER`] that can output characters to the 80×25
//! VGA text buffer at physical address `0xB8000`.

use core::fmt;
use core::ptr::NonNull;
use minios_common::types::{Color, ColorCode};
use spin::{Lazy, Mutex};
use volatile::VolatilePtr;

/// VGA buffer width in columns.
const BUFFER_WIDTH: usize = 80;

/// VGA buffer height in rows.
const BUFFER_HEIGHT: usize = 25;

/// Physical base address of the VGA text buffer.
const VGA_BUFFER_ADDR: usize = 0xB8000;

/// A single character cell in the VGA text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    /// The ASCII code point.
    pub ascii_character: u8,
    /// Packed foreground/background colour.
    pub color_code: ColorCode,
}

/// The raw VGA text buffer — a 2-D array of [`ScreenChar`] cells.
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer that outputs characters to the VGA text buffer.
pub struct Writer {
    /// Current column (0-based).
    column_position: usize,
    /// Current row (0-based).
    row_position: usize,
    /// Active foreground/background colour pair.
    color_code: ColorCode,
    /// Raw pointer to the memory-mapped VGA buffer.
    buffer: *mut Buffer,
}

// SAFETY: The buffer pointer targets a fixed hardware address and is
// only ever accessed through the Writer, which is protected by a Mutex.
unsafe impl Send for Writer {}

impl Writer {
    /// Returns a [`VolatilePtr`] to one character cell.
    fn cell(&self, row: usize, col: usize) -> VolatilePtr<'_, ScreenChar> {
        // SAFETY: VGA buffer is always valid, and indices are checked by
        // callers to stay within BUFFER_HEIGHT × BUFFER_WIDTH.
        unsafe {
            let ptr = core::ptr::addr_of_mut!((*self.buffer).chars[row][col]);
            VolatilePtr::new(NonNull::new_unchecked(ptr))
        }
    }

    /// Writes a single byte at the current cursor position.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let sc = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.cell(self.row_position, self.column_position).write(sc);
                self.column_position += 1;
            }
        }
    }

    /// Writes a string, replacing non-printable characters with `■` (0xFE).
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Advances the cursor to the beginning of the next line,
    /// scrolling the screen up if necessary.
    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            self.scroll_up();
        }
        self.column_position = 0;
    }

    /// Scrolls all rows up by one and clears the bottom row.
    fn scroll_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.cell(row, col).read();
                self.cell(row - 1, col).write(ch);
            }
        }
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.cell(BUFFER_HEIGHT - 1, col).write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global VGA text writer protected by a spinlock.
pub static WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| {
    Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: VGA_BUFFER_ADDR as *mut Buffer,
    })
});

/// Writes formatted arguments to the VGA buffer (implementation detail).
///
/// Interrupts are disabled for the duration to prevent deadlocks.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).expect("vga print failed");
    });
}

/// Prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!($($arg)*))
    };
}

/// Prints to the VGA text buffer, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
