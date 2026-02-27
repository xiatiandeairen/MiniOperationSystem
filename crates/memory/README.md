# minios-memory — OS Memory Management

Physical frame allocator (bitmap), virtual memory (4-level page tables), and kernel heap (linked list).

Note: This crate requires x86_64 hardware abstractions and is not yet standalone-ready. The bitmap allocator algorithm is reusable — see `frame.rs`.
