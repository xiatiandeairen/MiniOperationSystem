//! MiniOS Trace Engine — ring-buffer-backed span tracing for a bare-metal kernel.
//!
//! This crate provides:
//!
//! * [`Span`] — the core data structure for a single traced operation
//! * [`RingBuffer`] — lock-free fixed-size storage for spans
//! * [`TraceEngine`] — the [`Tracer`] implementation wired to a global static
//! * [`SpanGuard`] — RAII helper that ends a span on drop
//! * [`NullTracer`] — no-op tracer for tests
//! * [`export_json`] — hand-rolled JSON serialiser for `&[Span]`
//! * [`trace_span!`] / [`trace_event!`] — convenience macros

#![no_std]

/// Per-CPU trace context stack for parent/child span linking.
pub mod context;
/// Core trace engine implementing the `Tracer` trait.
pub mod engine;
/// Hand-rolled JSON serialiser for span data.
pub mod export;
/// RAII span guard that ends a span on drop.
pub mod guard;
/// Convenience macros (`trace_span!`, `trace_event!`).
pub mod macros;
/// No-op tracer for tests and disabled-tracing contexts.
pub mod null;
/// Fixed-size ring buffer storage for span records.
pub mod ringbuffer;
/// Span data structure representing a single traced operation.
pub mod span;

pub use engine::TRACER;
pub use export::export_json;
pub use guard::SpanGuard;
pub use null::NullTracer;
pub use ringbuffer::RingBuffer;
pub use span::Span;
