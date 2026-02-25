//! I/O system calls — read and write to file descriptors.
//!
//! For now, fd 1 (stdout) writes to the serial port and fd 0 (stdin)
//! reads from the keyboard buffer. All other descriptors return `-1`.

/// Writes `len` bytes from the buffer at `buf_ptr` to file descriptor `fd`.
///
/// Currently only fd 1 (stdout → serial) is supported.
/// Returns the number of bytes written, or `-1` on error.
pub fn sys_write(fd: i32, buf_ptr: u64, len: u64) -> i64 {
    if fd != 1 {
        return -1;
    }
    let slice = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len as usize) };
    for &byte in slice {
        minios_hal::serial_print!("{}", byte as char);
    }
    len as i64
}

/// Reads up to `len` bytes from file descriptor `fd` into the buffer at `buf_ptr`.
///
/// Currently only fd 0 (stdin → keyboard buffer) is supported.
/// Returns the number of bytes actually read (may be 0 if no input is available).
pub fn sys_read(fd: i32, buf_ptr: u64, len: u64) -> i64 {
    if fd != 0 {
        return -1;
    }
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len as usize) };
    let mut count = 0usize;
    for slot in buf.iter_mut() {
        match minios_hal::keyboard::read_key() {
            Some(key) => {
                *slot = key;
                count += 1;
            }
            None => break,
        }
    }
    count as i64
}
