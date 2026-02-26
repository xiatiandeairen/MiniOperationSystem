//! Built-in performance benchmarks.
extern crate alloc;
use minios_common::traits::trace::Tracer;
use minios_hal::{cpu::read_tsc, println};

pub fn cmd_bench(args: &[&str]) {
    if args.is_empty() {
        bench_list();
        return;
    }
    match args[0] {
        "list" => bench_list(),
        "alloc" | "1" => bench_alloc(),
        "trace" | "2" => bench_trace(),
        "fs" | "3" => bench_fs(),
        _ => println!("Unknown benchmark. Try 'bench list'."),
    }
}

fn bench_list() {
    println!("=== Performance Benchmarks ===");
    println!("  1. alloc  — Heap allocation speed");
    println!("  2. trace  — Trace span overhead");
    println!("  3. fs     — File create/write/read/delete cycle");
}

fn bench_alloc() {
    println!("=== Benchmark: Heap Allocation ===");
    let start = read_tsc();
    for _ in 0..1000 {
        let v = alloc::vec![0u8; 64];
        core::hint::black_box(&v);
    }
    let end = read_tsc();
    let per_op = (end - start) / 1000;
    println!("1000 x 64-byte alloc+free: {} cycles/op", per_op);
}

fn bench_trace() {
    println!("=== Benchmark: Trace Span ===");
    let start = read_tsc();
    for _ in 0..1000 {
        let _s = minios_trace::trace_span!("bench", module = "bench");
    }
    let end = read_tsc();
    println!("1000 spans: {} cycles/span", (end - start) / 1000);
}

fn bench_fs() {
    use minios_common::traits::fs::FileSystem;
    use minios_common::types::OpenFlags;
    println!("=== Benchmark: FS Operations ===");
    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("FS not initialized");
            return;
        }
    };
    let start = read_tsc();
    for _ in 0..100u32 {
        let fd = vfs
            .open("/tmp/bench", OpenFlags::CREATE | OpenFlags::WRITE)
            .unwrap();
        vfs.write(fd, b"benchmark data").unwrap();
        vfs.close(fd).unwrap();
        vfs.unlink("/tmp/bench").ok();
    }
    drop(vfs_guard);
    let end = read_tsc();
    println!(
        "100 x create+write+close+delete: {} cycles/op",
        (end - start) / 100
    );
}
