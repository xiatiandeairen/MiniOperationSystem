//! Trace shell commands: trace list, tree, stats, clear, export, follow, filter.

extern crate alloc;

use core::sync::atomic::{AtomicUsize, Ordering};
use minios_common::traits::trace::Tracer;
use minios_hal::println;
use spin::Mutex;

static TRACE_MODULE_FILTER: Mutex<[u8; 32]> = Mutex::new([0u8; 32]);
static TRACE_FILTER_LEN: AtomicUsize = AtomicUsize::new(0);

/// Returns a brief teaching description for known span names.
fn describe_span(name: &str) -> &'static str {
    match name {
        "syscall" => "\u{2190} system call entry",
        "sys_open" => "\u{2190} opens a file by path",
        "sys_read" => "\u{2190} reads bytes from file descriptor",
        "sys_write" => "\u{2190} writes bytes to file descriptor",
        "sys_close" => "\u{2190} releases file descriptor",
        "sys_exit" => "\u{2190} terminates the calling process",
        "sys_getpid" => "\u{2190} returns current process ID",
        "sys_yield" => "\u{2190} yields CPU to scheduler",
        "sys_uptime" => "\u{2190} returns ticks since boot",
        "sys_meminfo" => "\u{2190} writes memory info to buffer",
        "vfs_open" => "\u{2190} VFS resolves path to inode",
        "vfs_read" => "\u{2190} VFS delegates read to driver",
        "vfs_write" => "\u{2190} VFS delegates write to driver",
        "vfs_close" => "\u{2190} VFS releases file descriptor",
        "vfs_mkdir" => "\u{2190} VFS creates directory inode",
        "vfs_stat" => "\u{2190} VFS retrieves inode metadata",
        "vfs_seek" => "\u{2190} VFS adjusts file offset",
        "vfs_rmdir" => "\u{2190} VFS removes directory",
        "vfs_unlink" => "\u{2190} VFS removes file",
        "memory_init" => "\u{2190} initializes frame allocator + page tables + heap",
        "kernel_boot" => "\u{2190} full kernel initialization sequence",
        _ => "",
    }
}

/// Dispatches trace sub-commands.
pub fn cmd_trace(args: &[&str]) {
    let sub = if args.is_empty() { "help" } else { args[0] };

    match sub {
        "list" => trace_list(),
        "tree" => trace_tree(),
        "stats" => {
            trace_stats();
            super::journey::mark(super::journey::STEP_TRACE_STATS);
        }
        "clear" => trace_clear(),
        "export" => trace_export(),
        "follow" => {
            trace_follow(&args[1..]);
            super::journey::mark(super::journey::STEP_TRACE_FOLLOW);
        }
        "filter" => trace_filter(&args[1..]),
        _ => println!("Usage: trace <list|tree|stats|clear|export|follow|filter>"),
    }
}

/// Sets the trace module filter.
fn set_trace_module_filter(module: &str) {
    let mut filter = TRACE_MODULE_FILTER.lock();
    let len = module.len().min(31);
    filter[..len].copy_from_slice(&module.as_bytes()[..len]);
    TRACE_FILTER_LEN.store(len, Ordering::Relaxed);
}

/// Returns `true` if the span passes the current module filter.
fn should_show_span(span: &minios_trace::Span) -> bool {
    let len = TRACE_FILTER_LEN.load(Ordering::Relaxed);
    if len == 0 {
        return true;
    }
    let filter = TRACE_MODULE_FILTER.lock();
    let filter_str = core::str::from_utf8(&filter[..len]).unwrap_or("");
    if filter_str == "all" {
        return true;
    }
    span.module_str() == filter_str
}

/// Handles the `trace filter` sub-command.
fn trace_filter(args: &[&str]) {
    if args.is_empty() {
        let len = TRACE_FILTER_LEN.load(Ordering::Relaxed);
        let current = if len == 0 {
            "all (no filtering)"
        } else {
            let filter = TRACE_MODULE_FILTER.lock();
            let s = core::str::from_utf8(&filter[..len]).unwrap_or("?");
            if s == "all" {
                "all (no filtering)"
            } else {
                // We can't return a reference to locked data, so print inline
                println!("Usage: trace filter <module|all>");
                println!("Filters trace list/tree to show only matching module.");
                println!("Current filter: '{}'", s);
                return;
            }
        };
        println!("Usage: trace filter <module|all>");
        println!("Filters trace list/tree to show only matching module.");
        println!("Current filter: {}", current);
        return;
    }
    let module = args[0];
    if module == "all" {
        set_trace_module_filter("");
        println!("Trace filter cleared \u{2014} showing all modules.");
    } else {
        set_trace_module_filter(module);
        println!("Trace filter set: only showing module '{}'", module);
    }
}

/// Shows the 10 most recent trace spans.
fn trace_list() {
    let mut buf: [minios_trace::Span; 10] = core::array::from_fn(|_| minios_trace::Span::default());
    let n = minios_trace::TRACER.read_recent(10, &mut buf);
    if n == 0 {
        println!("No trace spans recorded.");
        return;
    }
    println!(
        "{:<8} {:<20} {:<16} {}",
        "SPAN_ID", "NAME", "MODULE", "STATUS"
    );
    for span in &buf[..n] {
        if !should_show_span(span) {
            continue;
        }
        let status = match span.status {
            minios_common::types::SpanStatus::Ok => "OK",
            minios_common::types::SpanStatus::Error => "ERROR",
            minios_common::types::SpanStatus::InProgress => "IN_PROGRESS",
        };
        println!(
            "{:<8} {:<20} {:<16} {}",
            span.span_id,
            span.name_str(),
            span.module_str(),
            status,
        );
    }
}

/// Shows recent spans as an indented tree based on depth.
fn trace_tree() {
    let mut buf: [minios_trace::Span; 20] = core::array::from_fn(|_| minios_trace::Span::default());
    let n = minios_trace::TRACER.read_recent(20, &mut buf);
    if n == 0 {
        println!("No trace spans recorded.");
        return;
    }
    for span in &buf[..n] {
        if !should_show_span(span) {
            continue;
        }
        let indent = "  ".repeat(span.depth as usize);
        let duration = span.end_tsc.saturating_sub(span.start_tsc);
        let status = match span.status {
            minios_common::types::SpanStatus::Ok => "OK",
            minios_common::types::SpanStatus::Error => "ERR",
            minios_common::types::SpanStatus::InProgress => "...",
        };
        let desc = describe_span(span.name_str());
        if desc.is_empty() {
            println!(
                "{}[{}] {} ({} cycles) {}",
                indent,
                span.module_str(),
                span.name_str(),
                duration,
                status,
            );
        } else {
            println!(
                "{}[{}] {} ({} cycles) {} {}",
                indent,
                span.module_str(),
                span.name_str(),
                duration,
                status,
                desc,
            );
        }
    }
}

/// Clears the trace buffer, executes a command, then displays the resulting trace tree.
fn trace_follow(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: trace follow <command> [args...] [--filter <module>]");
        return;
    }

    let mut cmd_args_end = args.len();
    let mut filter_module: Option<&str> = None;
    if args.len() >= 3 && args[args.len() - 2] == "--filter" {
        filter_module = Some(args[args.len() - 1]);
        cmd_args_end = args.len() - 2;
    }

    minios_trace::TRACER.clear();

    let cmd_name = args[0];
    let cmd_args = if cmd_args_end > 1 {
        &args[1..cmd_args_end]
    } else {
        &[]
    };
    match super::find_command(cmd_name) {
        Some(command) => (command.handler)(cmd_args),
        None => {
            println!("trace follow: unknown command '{}'", cmd_name);
            return;
        }
    }

    println!("--- Trace for '{}' ---", cmd_name);
    let mut buf: [minios_trace::Span; 32] = core::array::from_fn(|_| minios_trace::Span::default());
    let n = minios_trace::TRACER.read_recent(32, &mut buf);
    for span in &buf[..n] {
        if let Some(m) = filter_module {
            if span.module_str() != m {
                continue;
            }
        }
        let indent = "  ".repeat(span.depth as usize);
        let duration = span.end_tsc.saturating_sub(span.start_tsc);
        let desc = describe_span(span.name_str());
        if desc.is_empty() {
            println!(
                "{}[{}] {}  {} cycles",
                indent,
                span.module_str(),
                span.name_str(),
                duration,
            );
        } else {
            println!(
                "{}[{}] {}  {} cycles  {}",
                indent,
                span.module_str(),
                span.name_str(),
                duration,
                desc,
            );
        }
    }
    println!("--- {} spans ---", n);
}

/// Shows trace buffer statistics.
fn trace_stats() {
    let stats = minios_trace::TRACER.stats();
    println!("Total spans written: {}", stats.total_spans_written);
    println!("Buffer capacity:     {}", stats.buffer_capacity);
    println!("Buffer used:         {}", stats.buffer_used);
    println!("Active spans:        {}", stats.active_spans);
}

/// Clears the trace buffer.
fn trace_clear() {
    minios_trace::TRACER.clear();
    println!("Trace buffer cleared.");
}

/// Exports the trace buffer as JSON to the serial port.
///
/// The output can be captured with: `cargo make run-trace`
/// then loaded in the trace-viewer web tool.
fn trace_export() {
    let mut buf: [minios_trace::Span; 64] = core::array::from_fn(|_| minios_trace::Span::default());
    let n = minios_trace::TRACER.read_recent(64, &mut buf);
    if n == 0 {
        println!("No spans to export.");
        return;
    }
    println!("Exporting {} spans to serial port...", n);

    minios_hal::serial_println!("---TRACE-BEGIN---");
    minios_hal::serial_print!("{{\"format\":\"minios-trace-v1\",\"spans\":");
    let mut writer = minios_trace::export::SerialJsonWriter;
    let _ = minios_trace::export::export_json(&mut writer, &buf[..n]);
    minios_hal::serial_println!("}}");
    minios_hal::serial_println!("---TRACE-END---");

    println!("Export complete.");
    println!("Capture with: cargo make run-trace");
    println!("Then load trace-output.log in trace-viewer.");
}
