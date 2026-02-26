//! Trace shell commands: trace list, tree, stats, clear, export.

extern crate alloc;

use minios_common::traits::trace::Tracer;
use minios_hal::println;

/// Dispatches trace sub-commands.
pub fn cmd_trace(args: &[&str]) {
    let sub = if args.is_empty() { "help" } else { args[0] };

    match sub {
        "list" => trace_list(),
        "tree" => trace_tree(),
        "stats" => trace_stats(),
        "clear" => trace_clear(),
        "export" => trace_export(),
        _ => println!("Usage: trace <list|tree|stats|clear|export>"),
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
        println!(
            "{}[{}] {} ({} cycles) {}",
            indent,
            span.module_str(),
            span.name_str(),
            duration,
            status,
        );
    }
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
    let mut writer = minios_trace::export::SerialJsonWriter;
    let _ = minios_trace::export::export_json(&mut writer, &buf[..n]);
    println!("Export complete. Capture serial output to load in trace-viewer.");
}
