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

pub mod context;
pub mod engine;
pub mod export;
pub mod guard;
pub mod macros;
pub mod null;
pub mod ringbuffer;
pub mod span;

pub use engine::TRACER;
pub use export::export_json;
pub use guard::SpanGuard;
pub use null::NullTracer;
pub use ringbuffer::RingBuffer;
pub use span::Span;
