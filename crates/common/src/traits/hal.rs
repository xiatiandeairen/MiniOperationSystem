//! Hardware abstraction layer contracts.

use crate::types::ColorCode;

/// Serial port read/write capability.
pub trait HalSerial: Send + Sync {
    /// Sends a single byte over the serial port.
    fn write_byte(&self, byte: u8);
    /// Sends a slice of bytes over the serial port.
    fn write_bytes(&self, bytes: &[u8]);
    /// Reads one byte from the serial port, if available.
    fn read_byte(&self) -> Option<u8>;
}

/// Text-mode display output capability.
pub trait HalDisplay: Send + Sync {
    /// Writes a character at the given row/column with the specified color.
    fn write_char(&self, row: usize, col: usize, ch: u8, color: ColorCode);
    /// Scrolls the display up by one line.
    fn scroll_up(&self);
    /// Clears the entire display.
    fn clear(&self);
    /// Returns (rows, columns) dimensions.
    fn dimensions(&self) -> (usize, usize);
}

/// Interrupt controller hardware management.
pub trait HalInterruptController: Send + Sync {
    /// Initializes the interrupt controller.
    fn init(&self);
    /// Enables a specific IRQ line.
    fn enable_irq(&self, irq: u8);
    /// Disables a specific IRQ line.
    fn disable_irq(&self, irq: u8);
    /// Signals end-of-interrupt for the given IRQ.
    fn end_of_interrupt(&self, irq: u8);
}
