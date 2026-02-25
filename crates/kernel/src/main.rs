#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use minios_common::traits::memory::{FrameAllocator, HeapAllocator};
use minios_common::traits::trace::Tracer;

/// Map the complete physical memory so we can access VGA buffer at 0xB8000.
const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    minios_hal::gdt::init();
    minios_hal::interrupts::init_idt();
    minios_hal::serial::init();

    let _boot_guard = minios_trace::trace_span!("kernel_boot", module = "boot");

    minios_hal::serial_println!("MiniOS: boot sequence started");

    {
        let _hal_guard = minios_trace::trace_span!("hal_init", module = "boot");
        if let Some(phys_offset) = boot_info.physical_memory_offset.as_ref() {
            minios_hal::serial_println!("Physical memory offset: {:#x}", phys_offset);
            minios_hal::vga::init_with_offset(*phys_offset);
        }
    }

    minios_hal::println!("Hello, MiniOS!");

    let mem = {
        let _mem_guard = minios_trace::trace_span!("memory_init", module = "boot");
        minios_memory::init(boot_info).expect("memory init failed")
    };

    minios_hal::serial_println!(
        "Memory: total frames = {}, free frames = {}",
        mem.frame_allocator.total_frame_count(),
        mem.frame_allocator.free_frame_count()
    );
    minios_hal::serial_println!(
        "Heap: used = {} bytes, free = {} bytes",
        mem.heap.used_bytes(),
        mem.heap.free_bytes()
    );

    let v = vec![1, 2, 3];
    minios_hal::serial_println!("heap works: {:?}", v);

    minios_hal::println!("Boot successful. System ready.");
    minios_hal::serial_println!("MiniOS: VGA output written");

    minios_hal::enable_interrupts();
    minios_hal::serial_println!("MiniOS: interrupts enabled");

    let stats = minios_trace::TRACER.stats();
    minios_hal::serial_println!(
        "Trace: {} spans written, buffer {}/{}",
        stats.total_spans_written,
        stats.buffer_used,
        stats.buffer_capacity
    );

    minios_hal::cpu::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    minios_hal::serial_println!("KERNEL PANIC: {}", info);
    minios_hal::cpu::hlt_loop();
}
