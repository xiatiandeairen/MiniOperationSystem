//! PS/2 keyboard input buffer with scancode-to-ASCII translation.
//!
//! Stores keystrokes in a 256-byte circular buffer that interrupt handlers
//! push into and consumers drain via [`read_key`].

use spin::Mutex;

/// Size of the circular keystroke buffer in bytes.
const BUFFER_SIZE: usize = 256;

/// Global keyboard input buffer.
static KEYBOARD_BUFFER: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());

/// Tracks whether the left or right Shift key is currently held.
static SHIFT_STATE: Mutex<bool> = Mutex::new(false);

/// Circular buffer holding ASCII characters produced by the keyboard ISR.
struct KeyboardBuffer {
    buf: [u8; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl KeyboardBuffer {
    /// Creates an empty buffer.
    const fn new() -> Self {
        Self {
            buf: [0; BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    /// Pushes one byte. Silently drops the byte if the buffer is full.
    fn push(&mut self, byte: u8) {
        if self.count >= BUFFER_SIZE {
            return;
        }
        self.buf[self.write_pos] = byte;
        self.write_pos = (self.write_pos + 1) % BUFFER_SIZE;
        self.count += 1;
    }

    /// Pops the oldest byte, or returns `None` if empty.
    fn pop(&mut self) -> Option<u8> {
        if self.count == 0 {
            return None;
        }
        let byte = self.buf[self.read_pos];
        self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;
        self.count -= 1;
        Some(byte)
    }
}

/// Non-blocking read: returns the next ASCII character or `None`.
pub fn read_key() -> Option<u8> {
    x86_64::instructions::interrupts::without_interrupts(|| KEYBOARD_BUFFER.lock().pop())
}

/// Scancode Set 1 unshifted lookup table (index = scancode).
///
/// `0` means no printable character for that scancode.
static SCANCODE_MAP: [u8; 58] = [
    0, 0, // 0x00, 0x01 (Esc)
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', // 0x02–0x0B
    b'-', b'=', 0x08,  // 0x0C (minus), 0x0D (equals), 0x0E (Backspace)
    b'\t', // 0x0F (Tab)
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', // 0x10–0x19
    b'[', b']', b'\n', // 0x1A, 0x1B, 0x1C (Enter)
    0,     // 0x1D (LCtrl)
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', // 0x1E–0x26
    b';', b'\'', b'`',  // 0x27, 0x28, 0x29
    0,     // 0x2A (LShift — handled separately)
    b'\\', // 0x2B
    b'z', b'x', b'c', b'v', b'b', b'n', b'm', // 0x2C–0x32
    b',', b'.', b'/', // 0x33, 0x34, 0x35
    0,    // 0x36 (RShift — handled separately)
    0,    // 0x37 (Keypad *)
    0,    // 0x38 (LAlt)
    b' ', // 0x39 (Space)
];

/// Shifted equivalents for the same scancode positions.
static SCANCODE_MAP_SHIFT: [u8; 58] = [
    0, 0, // 0x00, 0x01
    b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', // 0x02–0x0B
    b'_', b'+', 0x08,  // 0x0C, 0x0D, 0x0E
    b'\t', // 0x0F
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', // 0x10–0x19
    b'{', b'}', b'\n', // 0x1A, 0x1B, 0x1C
    0,     // 0x1D
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', // 0x1E–0x26
    b':', b'"', b'~', // 0x27, 0x28, 0x29
    0,    // 0x2A
    b'|', // 0x2B
    b'Z', b'X', b'C', b'V', b'B', b'N', b'M', // 0x2C–0x32
    b'<', b'>', b'?', // 0x33, 0x34, 0x35
    0,    // 0x36
    0,    // 0x37
    0,    // 0x38
    b' ', // 0x39
];

/// Called from the keyboard interrupt handler to process a raw scancode.
///
/// Updates the Shift modifier state and, for key-press events that map to a
/// printable ASCII character, pushes the character into the global buffer.
pub fn handle_scancode(scancode: u8) {
    let is_release = scancode & 0x80 != 0;
    let key = scancode & 0x7F;

    // Shift press / release
    if key == 0x2A || key == 0x36 {
        *SHIFT_STATE.lock() = !is_release;
        return;
    }

    if is_release {
        return;
    }

    if let Some(&ch) = lookup_ascii(key) {
        if ch != 0 {
            KEYBOARD_BUFFER.lock().push(ch);
        }
    }
}

/// Looks up the ASCII byte for a given scancode, respecting Shift state.
fn lookup_ascii(key: u8) -> Option<&'static u8> {
    let idx = key as usize;
    if idx >= SCANCODE_MAP.len() {
        return None;
    }
    if *SHIFT_STATE.lock() {
        Some(&SCANCODE_MAP_SHIFT[idx])
    } else {
        Some(&SCANCODE_MAP[idx])
    }
}
