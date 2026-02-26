//! Memory exploration shell commands: pagetable, frames, alloc.

extern crate alloc;

use minios_hal::println;

/// Decomposes a virtual address into its 4-level page table indices.
pub fn cmd_pagetable(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: pagetable <hex_address>");
        return;
    }
    let addr = parse_hex(args[0]);
    if addr == 0 {
        println!("Invalid address");
        return;
    }

    let pml4_idx = (addr >> 39) & 0x1FF;
    let pdpt_idx = (addr >> 30) & 0x1FF;
    let pd_idx = (addr >> 21) & 0x1FF;
    let pt_idx = (addr >> 12) & 0x1FF;
    let offset = addr & 0xFFF;

    println!("Virtual address: {:#x}", addr);
    println!("  PML4 index: {} ({:#x})", pml4_idx, pml4_idx);
    println!("  PDPT index: {} ({:#x})", pdpt_idx, pdpt_idx);
    println!("  PD index:   {} ({:#x})", pd_idx, pd_idx);
    println!("  PT index:   {} ({:#x})", pt_idx, pt_idx);
    println!("  Offset:     {} ({:#x})", offset, offset);
    println!("(Use serial debug for actual translation)");
    super::journey::mark(super::journey::STEP_PAGETABLE);
}

/// Displays physical frame usage with a visual bar.
pub fn cmd_frames(_args: &[&str]) {
    let stats = minios_memory::get_stats();
    let used = stats.total_frames - stats.free_frames;
    let pct = (used * 100).checked_div(stats.total_frames).unwrap_or(0);

    let bar_filled = pct / 2;
    let bar_empty = 50 - bar_filled;
    minios_hal::print!("[");
    for _ in 0..bar_filled {
        minios_hal::print!("#");
    }
    for _ in 0..bar_empty {
        minios_hal::print!(".");
    }
    println!("]");
    println!(
        "{}% used ({}/{} frames, {} KiB free)",
        pct,
        used,
        stats.total_frames,
        stats.free_frames * 4,
    );
    super::journey::mark(super::journey::STEP_FRAMES);
}

/// Allocates a block of heap memory and reports the result.
pub fn cmd_alloc(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: alloc <size_bytes>");
        return;
    }
    let size = parse_u32(args[0]).unwrap_or(0) as usize;
    if size == 0 || size > 1_000_000 {
        println!("Invalid size (1-1000000)");
        return;
    }

    let layout = match alloc::alloc::Layout::from_size_align(size, 8) {
        Ok(l) => l,
        Err(_) => {
            println!("Invalid layout");
            return;
        }
    };
    let ptr = unsafe { alloc::alloc::alloc(layout) };
    if ptr.is_null() {
        println!("Allocation failed (OOM)");
    } else {
        println!("Allocated {} bytes at {:#x}", size, ptr as u64);
        let stats = minios_memory::get_stats();
        println!("Heap: {} used / {} free", stats.heap_used, stats.heap_free);
    }
}

/// Prints an ASCII diagram of the x86-64 virtual memory layout plus stats.
pub fn cmd_memmap(_args: &[&str]) {
    let stats = minios_memory::get_stats();
    println!("=== Memory Layout (x86-64) ===");
    println!();
    println!("  0xFFFF_FFFF_FFFF_FFFF \u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}");
    println!("                        \u{2502}   Kernel Space       \u{2502}");
    println!("  0xFFFF_8000_0000_0000 \u{251c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2524}");
    println!("                        \u{2502}   (non-canonical)    \u{2502}");
    println!("  0x0000_8000_0000_0000 \u{251c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2524}");
    println!("                        \u{2502}   User Space         \u{2502}");
    println!(
        "  0x0000_4444_4444_0000 \u{2502}   \u{250c}\u{2500} Heap (1 MiB) \u{2500}\u{2510}\u{2502}"
    );
    println!("                        \u{2502}   \u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}\u{2502}");
    println!("  0x0000_0000_0000_1000 \u{2502}   (unmapped)         \u{2502}");
    println!(
        "  0x0000_0000_0000_0000 \u{2514}\u{2500}\u{2500} NULL guard page \u{2500}\u{2500}\u{2500}\u{2518}"
    );
    println!();
    println!(
        "  Physical: {} frames ({} KiB), {} free",
        stats.total_frames,
        stats.total_frames * 4,
        stats.free_frames
    );
    println!(
        "  Heap: {} used / {} free bytes",
        stats.heap_used, stats.heap_free
    );
}

/// Parses a hexadecimal string (with optional `0x`/`0X` prefix) into `u64`.
fn parse_hex(s: &str) -> u64 {
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    let mut result: u64 = 0;
    for b in s.bytes() {
        let digit = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            b'_' => continue,
            _ => return 0,
        };
        result = result.wrapping_mul(16).wrapping_add(digit as u64);
    }
    result
}

/// Parses a decimal string into `u32`.
fn parse_u32(s: &str) -> Option<u32> {
    let mut result: u32 = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(result)
}
