//! CPU-level utility functions.
//!
//! Also provides a deferred-reschedule mechanism: the timer ISR sets a
//! flag instead of performing a context switch directly (which would be
//! unsafe inside the x86-interrupt ABI frame). The actual switch happens
//! at the next `hlt()` return — a known safe yield point where no Mutex
//! is held.

use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

/// Flag set by the timer ISR when the scheduler decides a task switch
/// is needed. Cleared by `hlt()` after invoking the reschedule callback.
pub static NEED_RESCHEDULE: AtomicBool = AtomicBool::new(false);

/// PID of the task the scheduler wants to switch to (set together with
/// [`NEED_RESCHEDULE`]).
pub static NEXT_TASK_PID: AtomicU32 = AtomicU32::new(0);

/// Signature of the reschedule callback registered by the kernel.
pub type RescheduleCallback = fn();

static RESCHEDULE_CB: Mutex<Option<RescheduleCallback>> = Mutex::new(None);

/// Registers the function the kernel wants called when a deferred
/// context switch fires. Called once during kernel init.
pub fn set_reschedule_callback(cb: RescheduleCallback) {
    *RESCHEDULE_CB.lock() = Some(cb);
}

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

/// Executes a single `HLT` instruction, suspending the CPU until the
/// next interrupt fires, then returns control to the caller.
///
/// After waking, checks whether the scheduler requested a context
/// switch (via [`NEED_RESCHEDULE`]) and, if so, invokes the registered
/// reschedule callback outside the ISR where it is safe.
pub fn hlt() {
    x86_64::instructions::hlt();

    if NEED_RESCHEDULE.swap(false, Ordering::Acquire) {
        if let Some(cb) = *RESCHEDULE_CB.lock() {
            cb();
        }
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
