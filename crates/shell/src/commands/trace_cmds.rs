//! Trace shell commands: trace list, tree, stats, clear, export, follow.

extern crate alloc;

use minios_common::traits::trace::Tracer;
use minios_hal::println;

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
        _ => println!("Usage: trace <list|tree|stats|clear|export|follow>"),
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
        println!("Usage: trace follow <command> [args...]");
        return;
    }

    minios_trace::TRACER.clear();

    let cmd_name = args[0];
    let cmd_args = if args.len() > 1 { &args[1..] } else { &[] };
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
