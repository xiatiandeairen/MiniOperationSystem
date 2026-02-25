#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use minios_common::id::Pid;
use minios_common::traits::fs::FileSystem;
use minios_common::traits::memory::{FrameAllocator, HeapAllocator};
use minios_common::types::{OpenFlags, Priority, ProcessState, ScheduleDecision};

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

    init_filesystem();

    init_processes();

    minios_hal::println!("Boot successful. System ready.");
    minios_hal::serial_println!("MiniOS: VGA output written");

    minios_hal::interrupts::set_timer_callback(on_timer_tick);
    minios_hal::enable_interrupts();
    minios_hal::serial_println!("MiniOS: interrupts enabled, scheduler active");

    minios_hal::cpu::hlt_loop();
}

/// Initialises the VFS, creates default directories, and runs a smoke test.
fn init_filesystem() {
    let vfs = minios_fs::init();
    minios_hal::serial_println!("Filesystem initialized");

    let fd = vfs
        .open("/tmp/test.txt", OpenFlags::CREATE | OpenFlags::WRITE)
        .expect("fs: create failed");
    vfs.write(fd, b"Hello from MiniOS filesystem!")
        .expect("fs: write failed");
    vfs.close(fd).expect("fs: close failed");

    let fd = vfs
        .open("/tmp/test.txt", OpenFlags::READ)
        .expect("fs: open-read failed");
    let mut buf = [0u8; 64];
    let n = vfs.read(fd, &mut buf).expect("fs: read failed");
    vfs.close(fd).expect("fs: close failed");

    minios_hal::serial_println!(
        "fs read: {}",
        core::str::from_utf8(&buf[..n]).unwrap_or("<invalid utf8>")
    );
}

/// Creates the idle task (PID 0) and init task (PID 1), registers them
/// with the scheduler, and prints the initial process listing.
fn init_processes() {
    use minios_process::manager;
    use minios_scheduler::SCHEDULER;

    let idle_pid = manager::create_kernel_task("idle", idle_task, Priority::IDLE)
        .expect("failed to create idle task");
    minios_hal::serial_println!("Created idle task: PID {}", idle_pid);

    let init_pid = manager::create_kernel_task("init", init_task, Priority::HIGH)
        .expect("failed to create init task");
    minios_hal::serial_println!("Created init task: PID {}", init_pid);

    {
        let mut sched = SCHEDULER.lock();
        sched.add_task(idle_pid, Priority::IDLE);
        sched.add_task(init_pid, Priority::HIGH);
    }

    manager::set_current(idle_pid);

    print_process_list();
    minios_hal::serial_println!("Process manager initialized");
}

/// Prints the current process table to serial output.
fn print_process_list() {
    let procs = minios_process::manager::list_processes();
    minios_hal::serial_println!("--- Process List ({} tasks) ---", procs.len());
    for p in &procs {
        minios_hal::serial_println!(
            "  PID {} | {:?} | priority {} | cpu_time {}",
            p.pid,
            p.state,
            p.priority.0,
            p.cpu_time_ticks,
        );
    }
    minios_hal::serial_println!("--- End Process List ---");
}

/// Timer tick callback — drives the scheduler.
fn on_timer_tick() {
    let ticks = minios_hal::interrupts::tick_count();
    let current = minios_process::manager::current_pid();
    minios_process::manager::tick_cpu_time(current);

    let decision = minios_scheduler::SCHEDULER.lock().tick();

    match decision {
        ScheduleDecision::Continue => {}
        ScheduleDecision::Switch(next_pid) => {
            handle_switch(current, next_pid);
        }
        ScheduleDecision::Idle => {}
    }

    if ticks > 0 && ticks.is_multiple_of(50) {
        minios_hal::serial_println!("tick {} | current PID {}", ticks, current);
    }

    if ticks == 200 {
        print_process_list();
        let stats = minios_scheduler::SCHEDULER.lock().stats();
        minios_hal::serial_println!(
            "Scheduler stats: switches={}, ticks={}, idle={}",
            stats.total_switches,
            stats.total_ticks,
            stats.idle_ticks
        );
    }
}

/// Handles a cooperative context switch between two tasks.
fn handle_switch(old_pid: Pid, new_pid: Pid) {
    use minios_process::manager;

    let _ = manager::set_state(old_pid, ProcessState::Ready);
    let _ = manager::set_state(new_pid, ProcessState::Running);

    {
        let mut sched = minios_scheduler::SCHEDULER.lock();
        sched.add_task(old_pid, Priority::IDLE);
        let q = sched
            .stats()
            .queue_lengths
            .iter()
            .position(|&l| l > 0)
            .unwrap_or(0);
        sched.set_running(new_pid, q);
    }

    manager::set_current(new_pid);
}

/// Idle task — simply halts until the next interrupt.
fn idle_task() {
    minios_hal::cpu::hlt_loop();
}

/// Init task — prints a message, then enters a halt loop.
fn init_task() {
    minios_hal::serial_println!("init process (PID 1) running");
    minios_hal::cpu::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    minios_hal::serial_println!("KERNEL PANIC: {}", info);
    minios_hal::cpu::hlt_loop();
}
