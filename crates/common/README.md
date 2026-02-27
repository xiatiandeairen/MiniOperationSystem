# minios-common — Shared OS Kernel Types and Traits

`#![no_std]` compatible types, error definitions, and trait contracts for OS kernel development.

## Contents
- **ID types**: `Pid`, `FileDescriptor`, `InodeId`, `TraceId`, `SpanId`, `QueueId`, `ShmId`
- **Error types**: `MemoryError`, `ProcessError`, `FsError`, `IpcError`, `DriverError`, `TraceError`
- **Trait contracts**: `FrameAllocator`, `Tracer`, `FileSystem`, `Scheduler`, `ProcessTable`, etc.
- **Data types**: `ProcessState`, `Priority`, `OpenFlags`, `ColorCode`, `SpanStatus`, etc.

## Usage
```toml
[dependencies]
minios-common = "0.1"
```

No OS dependencies — pure types and traits. Works in any `#![no_std]` environment.
