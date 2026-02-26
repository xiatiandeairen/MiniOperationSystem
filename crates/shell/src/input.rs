//! Line input buffer for the interactive shell.

/// Maximum number of bytes in a single input line.
const MAX_LINE_LEN: usize = 256;

/// Fixed-size buffer holding the current input line.
pub struct LineBuffer {
    buf: [u8; MAX_LINE_LEN],
    len: usize,
}

impl LineBuffer {
    /// Creates an empty line buffer.
    pub const fn new() -> Self {
        Self {
            buf: [0; MAX_LINE_LEN],
            len: 0,
        }
    }

    /// Appends a printable ASCII character if space remains.
    pub fn push(&mut self, byte: u8) {
        if self.len >= self.buf.len() - 1 {
            return;
        }
        self.buf[self.len] = byte;
        self.len += 1;
    }

    /// Removes the last character (backspace behaviour).
    pub fn backspace(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    /// Resets the buffer to empty.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns the current content as a UTF-8 string slice.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
