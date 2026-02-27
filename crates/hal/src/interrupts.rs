//! Interrupt Descriptor Table (IDT) and hardware interrupt handling.
//!
//! Registers handlers for CPU exceptions (breakpoint, double fault,
//! page fault, general protection fault) and PIC-routed hardware
//! interrupts (timer, keyboard).

use crate::gdt;
use crate::pic::ChainedPics;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::{Lazy, Mutex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

/// Base vector offset for the master PIC.
pub const PIC_1_OFFSET: u8 = 32;
/// Base vector offset for the slave PIC.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Global cascaded PIC instance.
///
/// Protected by a [`Mutex`] to allow safe EOI signalling from interrupt
/// handlers.
pub static PICS: Mutex<ChainedPics> =
    // SAFETY: Offsets 32/40 do not collide with CPU exceptions (0–31).
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Named interrupt vector indices.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// PIT timer interrupt (IRQ 0).
    Timer = PIC_1_OFFSET,
    /// PS/2 keyboard interrupt (IRQ 1).
    Keyboard,
}

impl InterruptIndex {
    /// Returns the vector number as `u8`.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Returns the vector number as `usize` (for IDT indexing).
    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// System call interrupt vector (legacy int 0x80).
pub const SYSCALL_VECTOR: u8 = 0x80;

/// Counter for int 0x80 software interrupt traps.
static SYSCALL_TRAP_COUNT: AtomicU64 = AtomicU64::new(0);

/// Returns the number of int 0x80 syscall traps since boot.
pub fn syscall_trap_count() -> u64 {
    SYSCALL_TRAP_COUNT.load(Ordering::Relaxed)
}

/// Monotonic tick counter incremented by the timer interrupt handler.
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// Total number of timer interrupts received since boot.
static TIMER_COUNT: AtomicU64 = AtomicU64::new(0);

/// Total number of keyboard interrupts received since boot.
static KEYBOARD_COUNT: AtomicU64 = AtomicU64::new(0);

/// Whether the timer tick callback is installed.
static TIMER_CALLBACK_SET: AtomicBool = AtomicBool::new(false);

/// Optional callback invoked on every timer tick (after incrementing count).
static TIMER_CALLBACK: Mutex<Option<fn()>> = Mutex::new(None);

/// Returns the number of timer ticks since boot.
pub fn tick_count() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

/// Returns the total number of timer interrupts since boot.
pub fn timer_interrupt_count() -> u64 {
    TIMER_COUNT.load(Ordering::Relaxed)
}

/// Returns the total number of keyboard interrupts since boot.
pub fn keyboard_interrupt_count() -> u64 {
    KEYBOARD_COUNT.load(Ordering::Relaxed)
}

/// Snapshot of all interrupt counters.
pub struct InterruptStats {
    pub timer_count: u64,
    pub keyboard_count: u64,
}

/// Returns a consistent snapshot of all interrupt counters.
pub fn interrupt_stats() -> InterruptStats {
    InterruptStats {
        timer_count: TICK_COUNT.load(Ordering::Relaxed),
        keyboard_count: KEYBOARD_COUNT.load(Ordering::Relaxed),
    }
}

/// Registers a function to be called on every timer interrupt.
///
/// Only one callback is supported; later calls overwrite earlier ones.
pub fn set_timer_callback(cb: fn()) {
    *TIMER_CALLBACK.lock() = Some(cb);
    TIMER_CALLBACK_SET.store(true, Ordering::Release);
}

/// The Interrupt Descriptor Table, lazily initialised.
static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    // SAFETY: DOUBLE_FAULT_IST_INDEX is a valid IST entry configured in
    // the TSS by gdt::init().
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);

    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_handler);
    idt[SYSCALL_VECTOR].set_handler_fn(syscall_trap_handler);

    idt
});

/// Loads the IDT and initialises the chained PICs.
pub fn init_idt() {
    IDT.load();
    // SAFETY: Called once during early boot with interrupts disabled.
    unsafe { PICS.lock().initialize() };
}

// ── Exception Handlers ──────────────────────────────────────────────

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    crate::serial_println!("EXCEPTION: PAGE FAULT");
    crate::serial_println!("Error Code: {:?}", error_code);
    crate::serial_println!("{:#?}", stack_frame);
    crate::cpu::hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

// ── Hardware Interrupt Handlers ─────────────────────────────────────

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    TIMER_COUNT.fetch_add(1, Ordering::Relaxed);

    if TIMER_CALLBACK_SET.load(Ordering::Acquire) {
        if let Some(cb) = *TIMER_CALLBACK.lock() {
            cb();
        }
    }

    // SAFETY: We are inside the timer ISR for this vector.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    KEYBOARD_COUNT.fetch_add(1, Ordering::Relaxed);
    let mut port = Port::new(0x60);
    // SAFETY: Reading port 0x60 retrieves the keyboard scancode and
    // is required to acknowledge the keyboard interrupt.
    let scancode: u8 = unsafe { port.read() };

    crate::keyboard::handle_scancode(scancode);

    // SAFETY: We are inside the keyboard ISR for this vector.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

// ── Software Interrupt Handlers ─────────────────────────────────────

extern "x86-interrupt" fn syscall_trap_handler(_stack_frame: InterruptStackFrame) {
    SYSCALL_TRAP_COUNT.fetch_add(1, Ordering::Relaxed);
}
