//! CPU context for cooperative context switching.
//!
//! Stores the callee-saved registers that must be preserved across function
//! calls on x86-64 (System V ABI). The actual register save/restore is
//! performed by a small assembly stub.

use core::arch::global_asm;

/// Callee-saved register state for context switching.
///
/// The field order **must** match the assembly in `switch_context_asm`.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CpuContext {
    /// R15 general-purpose register.
    pub r15: u64,
    /// R14 general-purpose register.
    pub r14: u64,
    /// R13 general-purpose register.
    pub r13: u64,
    /// R12 general-purpose register.
    pub r12: u64,
    /// RBX general-purpose register.
    pub rbx: u64,
    /// RBP frame pointer.
    pub rbp: u64,
    /// RSP stack pointer.
    pub rsp: u64,
    /// RIP instruction pointer (return address).
    pub rip: u64,
}

impl CpuContext {
    /// Returns a zeroed context (suitable for overwriting before first use).
    pub const fn empty() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rsp: 0,
            rip: 0,
        }
    }

    /// Creates a context that will begin execution at `entry` using `stack_top`.
    pub fn new(entry: u64, stack_top: u64) -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rsp: stack_top,
            rip: entry,
        }
    }
}

// The assembly stub:
//   rdi = pointer to old CpuContext (save current state here)
//   rsi = pointer to new CpuContext (load state from here)
global_asm!(
    ".global switch_context_asm",
    "switch_context_asm:",
    // Save callee-saved regs into *rdi
    "mov [rdi + 0],  r15",
    "mov [rdi + 8],  r14",
    "mov [rdi + 16], r13",
    "mov [rdi + 24], r12",
    "mov [rdi + 32], rbx",
    "mov [rdi + 40], rbp",
    "mov [rdi + 48], rsp",
    "lea rax, [rip + .Lreturn]",
    "mov [rdi + 56], rax",
    // Load callee-saved regs from *rsi
    "mov r15, [rsi + 0]",
    "mov r14, [rsi + 8]",
    "mov r13, [rsi + 16]",
    "mov r12, [rsi + 24]",
    "mov rbx, [rsi + 32]",
    "mov rbp, [rsi + 40]",
    "mov rsp, [rsi + 48]",
    "jmp [rsi + 56]",
    ".Lreturn:",
    "ret",
);

extern "C" {
    /// Raw context-switch routine implemented in assembly.
    ///
    /// Saves the current callee-saved registers into `old` and restores
    /// them from `new`, then jumps to the new context's instruction pointer.
    ///
    /// # Safety
    ///
    /// Both pointers must be valid, properly aligned `CpuContext` values.
    /// The new context must have a valid stack and instruction pointer.
    fn switch_context_asm(old: *mut CpuContext, new: *const CpuContext);
}

/// Performs a cooperative context switch from `old` to `new`.
///
/// # Safety
///
/// Both pointers must reference valid, aligned `CpuContext` instances.
/// The target context must have a valid stack and code pointer.
pub unsafe fn switch_context(old: *mut CpuContext, new: *const CpuContext) {
    unsafe { switch_context_asm(old, new) }
}
