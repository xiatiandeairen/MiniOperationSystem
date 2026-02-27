//! Hand-rolled JSON export for trace spans.
//!
//! Avoids `serde` so the crate stays `#![no_std]`-compatible with zero
//! heap allocation.

use crate::span::Span;
use core::fmt::Write;

/// A [`Write`] adapter that sends output directly to the serial port.
///
/// Used by Shell `trace export` to dump JSON spans to a host-side capture.
/// Only available when the `hal` feature is enabled.
#[cfg(feature = "hal")]
pub struct SerialJsonWriter;

#[cfg(feature = "hal")]
impl Write for SerialJsonWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        minios_hal::serial::_serial_print(format_args!("{}", s));
        Ok(())
    }
}

/// Writes an array of spans as a JSON array to `writer`.
///
/// The output format follows the project spec: each span is an object with
/// `trace_id`, `span_id`, `parent_span_id`, `name`, `module`,
/// `start_tsc`, `end_tsc`, `status`, `pid`, and `depth` fields.
pub fn export_json(writer: &mut dyn Write, spans: &[Span]) -> core::fmt::Result {
    writer.write_str("[")?;
    for (i, span) in spans.iter().enumerate() {
        if i > 0 {
            writer.write_str(",")?;
        }
        write_span_json(writer, span)?;
    }
    writer.write_str("]")
}

/// Serialises a single [`Span`] as a JSON object.
fn write_span_json(w: &mut dyn Write, span: &Span) -> core::fmt::Result {
    w.write_str("{")?;
    write!(w, "\"trace_id\":\"{}\"", span.trace_id)?;
    write!(w, ",\"span_id\":\"{}\"", span.span_id)?;

    match span.parent_span_id {
        Some(pid) => write!(w, ",\"parent_span_id\":\"{}\"", pid)?,
        None => w.write_str(",\"parent_span_id\":null")?,
    }

    w.write_str(",\"name\":\"")?;
    write_escaped(w, span.name_str())?;
    w.write_str("\"")?;

    w.write_str(",\"module\":\"")?;
    write_escaped(w, span.module_str())?;
    w.write_str("\"")?;

    write!(w, ",\"start_tsc\":{}", span.start_tsc)?;
    write!(w, ",\"end_tsc\":{}", span.end_tsc)?;
    write!(w, ",\"status\":\"{}\"", status_str(span.status))?;
    write!(w, ",\"pid\":{}", span.pid)?;
    write!(w, ",\"depth\":{}", span.depth)?;
    w.write_str("}")
}

/// Writes a string with JSON-required escaping for `"` and `\`.
fn write_escaped(w: &mut dyn Write, s: &str) -> core::fmt::Result {
    for ch in s.chars() {
        match ch {
            '"' => w.write_str("\\\"")?,
            '\\' => w.write_str("\\\\")?,
            _ => w.write_char(ch)?,
        }
    }
    Ok(())
}

/// Maps a [`SpanStatus`] to its JSON string representation.
fn status_str(s: minios_common::types::SpanStatus) -> &'static str {
    match s {
        minios_common::types::SpanStatus::Ok => "ok",
        minios_common::types::SpanStatus::Error => "error",
        minios_common::types::SpanStatus::InProgress => "in_progress",
    }
}
