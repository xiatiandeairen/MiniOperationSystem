//! Syscall dispatcher — maps syscall numbers to handler functions.

use crate::{SYS_EXIT, SYS_GETPID, SYS_MEMINFO, SYS_READ, SYS_UPTIME, SYS_WRITE, SYS_YIELD};
use minios_common::traits::trace::Tracer;

/// Dispatches a system call by number to the appropriate handler.
///
/// # Arguments
/// * `num`  — syscall number
/// * `arg1` — first argument (meaning depends on syscall)
/// * `arg2` — second argument
/// * `arg3` — third argument
///
/// Returns the handler's result, or `-1` (ENOSYS) for unknown numbers.
pub fn dispatch(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let _span = minios_trace::trace_span!("syscall", module = "syscall");
    minios_hal::klog!(
        Debug,
        "syscall",
        "dispatch: num={}, args=[{},{},{}]",
        num,
        arg1,
        arg2,
        arg3
    );
    match num {
        SYS_READ => {
            let _s = minios_trace::trace_span!("sys_read", module = "syscall");
            crate::io::sys_read(arg1 as i32, arg2, arg3)
        }
        SYS_WRITE => {
            let _s = minios_trace::trace_span!("sys_write", module = "syscall");
            crate::io::sys_write(arg1 as i32, arg2, arg3)
        }
        SYS_EXIT => {
            let _s = minios_trace::trace_span!("sys_exit", module = "syscall");
            crate::process::sys_exit(arg1 as i64)
        }
        SYS_GETPID => {
            let _s = minios_trace::trace_span!("sys_getpid", module = "syscall");
            crate::process::sys_getpid()
        }
        SYS_YIELD => {
            let _s = minios_trace::trace_span!("sys_yield", module = "syscall");
            crate::process::sys_yield()
        }
        SYS_UPTIME => {
            let _s = minios_trace::trace_span!("sys_uptime", module = "syscall");
            crate::info::sys_uptime()
        }
        SYS_MEMINFO => {
            let _s = minios_trace::trace_span!("sys_meminfo", module = "syscall");
            crate::info::sys_meminfo(arg1, arg2)
        }
        _ => -1, // ENOSYS
    }
}
