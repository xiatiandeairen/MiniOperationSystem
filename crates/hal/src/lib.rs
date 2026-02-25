//! Hardware Abstraction Layer for MiniOS.
//!
//! Provides low-level drivers and initialisation routines for the
//! serial port, VGA text display, GDT/TSS, IDT, PIC interrupts, and
//! CPU utilities.

#![no_std]
#![feature(abi_x86_interrupt)]

pub mod cpu;
pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
mod pic;
pub mod serial;
pub mod vga;

/// Initialises all hardware subsystems in the correct order.
///
/// 1. GDT + TSS (needed before IDT for double-fault stack)
/// 2. IDT + PIC (exception/interrupt handlers)
/// 3. Serial port (COM1 UART)
///
/// Does **not** enable interrupts — call [`enable_interrupts`] after
/// completing all initialisation that depends on an interrupt-free context.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    serial::init();
}

/// Enables hardware interrupts via the `sti` instruction.
pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}
