#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use minios_common::traits::memory::{FrameAllocator, HeapAllocator};

/// Map the complete physical memory so we can access VGA buffer at 0xB8000.
/// Stack is increased from the default 80 KiB to 512 KiB to accommodate
/// debug-build stack frames during memory subsystem initialisation.
const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.kernel_stack_size = 512 * 1024;
    config
};

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    minios_hal::gdt::init();
    minios_hal::interrupts::init_idt();
    minios_hal::serial::init();

    minios_hal::serial_println!("MiniOS: boot sequence started");

    if let Some(phys_offset) = boot_info.physical_memory_offset.as_ref() {
        minios_hal::serial_println!("Physical memory offset: {:#x}", phys_offset);
        minios_hal::vga::init_with_offset(*phys_offset);
    }

    minios_hal::println!("Hello, MiniOS!");

    let mem = minios_memory::init(boot_info).expect("memory init failed");

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

    minios_hal::cpu::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    minios_hal::serial_println!("KERNEL PANIC: {}", info);
    minios_hal::cpu::hlt_loop();
}
