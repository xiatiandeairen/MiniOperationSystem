//! CPU-level utility functions.

use core::arch::asm;

/// Enters a halt loop, reducing CPU usage until the next interrupt.
///
/// Each iteration executes `HLT`, which suspends the CPU until the next
/// interrupt fires. The surrounding loop ensures the CPU halts again
/// after the interrupt is serviced.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Reads the CPU Time Stamp Counter via the `RDTSC` instruction.
///
/// Returns a monotonically increasing 64-bit tick count. The counter
/// frequency is CPU-dependent and not guaranteed to be constant across
/// power-state transitions.
pub fn read_tsc() -> u64 {
    let lo: u32;
    let hi: u32;
    // SAFETY: `rdtsc` is always available on x86_64 and has no side
    // effects beyond reading a monotonic counter.
    unsafe {
        asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    }
    (hi as u64) << 32 | lo as u64
}
