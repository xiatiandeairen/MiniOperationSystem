#![no_std]
#![no_main]

use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;

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

    minios_hal::serial_println!("MiniOS: boot sequence started");

    if let Some(phys_offset) = boot_info.physical_memory_offset.as_ref() {
        minios_hal::serial_println!("Physical memory offset: {:#x}", phys_offset);
        minios_hal::vga::init_with_offset(*phys_offset);
    }

    minios_hal::println!("Hello, MiniOS!");
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
