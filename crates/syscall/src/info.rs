//! Informational system calls.

use core::fmt::Write;

/// Returns the number of timer ticks since boot.
pub fn sys_uptime() -> i64 {
    minios_hal::interrupts::tick_count() as i64
}

/// Writes a human-readable memory-info string into the buffer at `buf_ptr`.
///
/// At most `len` bytes are written. Returns the number of bytes written,
/// or `-1` if the buffer pointer is null.
pub fn sys_meminfo(buf_ptr: u64, len: u64) -> i64 {
    if buf_ptr == 0 {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len as usize) };
    let mut writer = BufWriter::new(buf);
    let _ = write!(writer, "MiniOS memory info (placeholder)");
    writer.written() as i64
}

/// Minimal fixed-capacity buffer writer for formatting into `&mut [u8]`.
struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn written(&self) -> usize {
        self.pos
    }
}

impl Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let to_copy = bytes.len().min(remaining);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}
