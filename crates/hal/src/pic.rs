//! Minimal 8259 PIC (Programmable Interrupt Controller) driver.
//!
//! Implements the standard two-chip cascaded PIC configuration found on
//! IBM PC compatibles, using I/O port access from the `x86_64` crate.

use x86_64::instructions::port::Port;

/// ICW1: initialisation command, ICW4 needed.
const CMD_INIT: u8 = 0x11;
/// OCW2: non-specific end-of-interrupt.
const CMD_END_OF_INTERRUPT: u8 = 0x20;
/// ICW4: 8086/88 mode.
const MODE_8086: u8 = 0x01;

/// A single 8259 PIC chip.
struct Pic {
    offset: u8,
    command_port: u16,
    data_port: u16,
}

impl Pic {
    /// Returns `true` if this chip handles the given interrupt vector.
    const fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }

    /// Sends the EOI command to this PIC.
    unsafe fn end_of_interrupt(&self) {
        // SAFETY: Writing the EOI command to the command port is the
        // documented way to acknowledge an interrupt.
        unsafe { Port::new(self.command_port).write(CMD_END_OF_INTERRUPT) }
    }
}

/// A pair of cascaded 8259 PICs (master + slave).
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    /// Creates a new `ChainedPics` with the given vector offsets.
    ///
    /// # Safety
    ///
    /// The offsets must not collide with CPU exception vectors (0–31).
    pub const unsafe fn new(offset1: u8, offset2: u8) -> Self {
        Self {
            pics: [
                Pic {
                    offset: offset1,
                    command_port: 0x20,
                    data_port: 0x21,
                },
                Pic {
                    offset: offset2,
                    command_port: 0xA0,
                    data_port: 0xA1,
                },
            ],
        }
    }

    /// Runs the full ICW1–ICW4 initialisation sequence.
    ///
    /// # Safety
    ///
    /// Must only be called once during early boot, with interrupts disabled.
    pub unsafe fn initialize(&mut self) {
        // SAFETY: All port accesses follow the 8259 PIC init protocol.
        unsafe {
            let mut wait: Port<u8> = Port::new(0x80);

            let saved_mask0: u8 = Port::new(self.pics[0].data_port).read();
            let saved_mask1: u8 = Port::new(self.pics[1].data_port).read();

            Port::new(self.pics[0].command_port).write(CMD_INIT);
            wait.write(0);
            Port::new(self.pics[1].command_port).write(CMD_INIT);
            wait.write(0);

            Port::new(self.pics[0].data_port).write(self.pics[0].offset);
            wait.write(0);
            Port::new(self.pics[1].data_port).write(self.pics[1].offset);
            wait.write(0);

            Port::new(self.pics[0].data_port).write(4u8); // slave on IRQ2
            wait.write(0);
            Port::new(self.pics[1].data_port).write(2u8); // cascade identity
            wait.write(0);

            Port::new(self.pics[0].data_port).write(MODE_8086);
            wait.write(0);
            Port::new(self.pics[1].data_port).write(MODE_8086);
            wait.write(0);

            Port::new(self.pics[0].data_port).write(saved_mask0);
            Port::new(self.pics[1].data_port).write(saved_mask1);
        }
    }

    /// Returns `true` if either PIC handles the given interrupt vector.
    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics[0].handles_interrupt(interrupt_id) || self.pics[1].handles_interrupt(interrupt_id)
    }

    /// Sends the appropriate EOI command(s) for the given interrupt.
    ///
    /// # Safety
    ///
    /// Must only be called from within an interrupt handler for the
    /// given vector.
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.pics[1].handles_interrupt(interrupt_id) {
            // SAFETY: Slave EOI must be sent before master EOI.
            unsafe { self.pics[1].end_of_interrupt() };
        }
        if self.handles_interrupt(interrupt_id) {
            // SAFETY: Master EOI acknowledges the cascaded interrupt.
            unsafe { self.pics[0].end_of_interrupt() };
        }
    }
}
