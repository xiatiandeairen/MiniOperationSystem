# minios-trace — Lightweight no_std Trace Engine

A zero-allocation span-based tracing library for `#![no_std]` environments.

## Features

- **4096-slot ring buffer** — fixed size, no heap allocation in the hot path
- **SpanGuard RAII** — automatic span lifecycle via `Drop`
- **Nested span context propagation** — parent/child linking through a per-CPU context stack
- **JSON export** — hand-rolled serialiser (no `serde` dependency)
- **TSC-based nanosecond timing** — uses `rdtsc` on x86_64, falls back to 0 on other architectures

## Core Types

| Type | Module | Purpose |
|------|--------|---------|
| `Span` | `span` | Fixed-size record for a single traced operation |
| `RingBuffer` | `ringbuffer` | Circular buffer storing up to 4096 spans |
| `TraceEngine` | `engine` | Global `Tracer` implementation managing span lifecycle |
| `SpanGuard` | `guard` | RAII wrapper that ends a span on drop |
| `NullTracer` | `null` | No-op tracer for tests and disabled-tracing contexts |
| `TraceContextStack` | `context` | Per-CPU stack tracking active trace/span IDs |

## Usage

```rust
// Start a span (automatically ended when guard is dropped)
let _guard = trace_span!("alloc_page", module = "memory");

// Record an instant event
trace_event!("page_fault");

// Export recent spans as JSON
let mut out = [Span::default(); 64];
let n = TRACER.read_recent(64, &mut out);
let mut writer = SerialJsonWriter;
export_json(&mut writer, &out[..n]).ok();
```

## Dependencies

- `minios-common` — shared types (`TraceId`, `SpanId`, `SpanStatus`, `Tracer` trait)
- `minios-hal` — serial port output (used only by `SerialJsonWriter`)
- `spin` — mutex for ring buffer and context stack

## Design Notes

The ring buffer overwrites the oldest entry when full, so long-running traces
gracefully degrade rather than failing. All span data uses fixed-size byte
arrays to avoid heap allocation on the write path.

Context propagation uses a 64-deep stack of `TraceContext` frames protected by
a spin lock. Nested `begin_span` / `end_span` calls automatically link
parent ↔ child spans via the context stack.
