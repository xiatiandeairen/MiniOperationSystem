//! Hardware abstraction layer contracts.

use crate::types::ColorCode;

/// Serial port read/write capability.
pub trait HalSerial: Send + Sync {
    fn write_byte(&self, byte: u8);
    fn write_bytes(&self, bytes: &[u8]);
    fn read_byte(&self) -> Option<u8>;
}

/// Text-mode display output capability.
pub trait HalDisplay: Send + Sync {
    fn write_char(&self, row: usize, col: usize, ch: u8, color: ColorCode);
    fn scroll_up(&self);
    fn clear(&self);
    fn dimensions(&self) -> (usize, usize);
}

/// Interrupt controller hardware management.
pub trait HalInterruptController: Send + Sync {
    fn init(&self);
    fn enable_irq(&self, irq: u8);
    fn disable_irq(&self, irq: u8);
    fn end_of_interrupt(&self, irq: u8);
}
