#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use minios_common::id::Pid;
use minios_common::traits::fs::FileSystem;
use minios_common::traits::memory::{FrameAllocator, HeapAllocator};
use minios_common::traits::trace::Tracer;
use minios_common::types::{OpenFlags, Priority, ProcessState, ScheduleDecision};
use minios_trace::{trace_event, trace_span};

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
    // HAL init (before trace is usable)
    minios_hal::gdt::init();
    minios_hal::interrupts::init_idt();
    minios_hal::serial::init();

    minios_hal::serial_println!("MiniOS: boot sequence started");

    // Extract framebuffer info before memory init borrows boot_info.
    // We take the raw pointer to the framebuffer buffer and reconstruct
    // a slice so that boot_info's borrow is not held when memory::init runs.
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        let info = fb.info();
        let buf = fb.buffer_mut();
        let ptr = buf.as_mut_ptr();
        let len = buf.len();
        // SAFETY: The framebuffer memory is mapped for the entire kernel
        // lifetime by the bootloader and not aliased by any other code.
        let static_buf: &'static mut [u8] = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        unsafe {
            minios_hal::framebuffer::init(
                static_buf,
                info.width,
                info.height,
                info.bytes_per_pixel,
                info.stride,
            );
        }
        minios_hal::serial_println!(
            "Framebuffer: {}x{} px, {}x{} chars",
            info.width,
            info.height,
            info.width / 8,
            info.height / 16,
        );
    }

    let _boot_span = trace_span!("kernel_boot", module = "boot");

    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::GREEN);
    minios_hal::println!("  __  __ _       _  ___  ____");
    minios_hal::println!(" |  \\/  (_)_ __ (_)/ _ \\/ ___|");
    minios_hal::println!(" | |\\/| | | '_ \\| | | | \\___ \\");
    minios_hal::println!(" | |  | | | | | | | |_| |___) |");
    minios_hal::println!(" |_|  |_|_|_| |_|_|\\___/|____/");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    minios_hal::println!();

    let mem = {
        let _mem_span = trace_span!("memory_init", module = "memory");
        let m = minios_memory::init(boot_info).expect("memory init failed");
        minios_hal::serial_println!(
            "Memory: total frames = {}, free frames = {}",
            m.frame_allocator.total_frame_count(),
            m.frame_allocator.free_frame_count()
        );
        minios_hal::serial_println!(
            "Heap: used = {} bytes, free = {} bytes",
            m.heap.used_bytes(),
            m.heap.free_bytes()
        );
        m
    };

    let v = vec![1, 2, 3];
    minios_hal::serial_println!("heap works: {:?}", v);

    mem.publish_stats();

    {
        let _fs_span = trace_span!("filesystem_init", module = "fs");
        init_filesystem();
    }

    {
        let _syscall_span = trace_span!("syscall_test", module = "syscall");
        test_syscalls();
    }

    {
        let _ipc_span = trace_span!("ipc_test", module = "ipc");
        test_ipc();
    }

    {
        let _proc_span = trace_span!("process_init", module = "process");
        init_processes();
    }

    minios_hal::println!("Boot successful. System ready.");
    minios_hal::serial_println!("MiniOS: VGA output written");

    minios_hal::interrupts::set_timer_callback(on_timer_tick);
    minios_hal::enable_interrupts();
    minios_hal::serial_println!("MiniOS: interrupts enabled, scheduler active");

    minios_shell::run_shell();
}

/// Initialises the VFS, creates default directories, and runs a smoke test.
/// Stores the VFS in the global static for shell access.
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

    minios_fs::set_global_vfs(vfs);
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
    trace_event!("timer_tick");
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

/// Records a scheduling decision without performing a real context switch.
///
/// Real context switching (via `switch_context_asm`) is available but
/// currently the Shell runs in `kernel_main` which is not a registered
/// process. Preempting it would lose the Shell execution context.
/// Instead we track scheduling decisions for observability while the
/// Shell remains the active execution path.
///
/// When process isolation is added (future work), tasks will be switched
/// using `minios_process::context::switch_context`.
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

/// Idle task — halts between interrupts, resumes when scheduled back.
fn idle_task() {
    loop {
        minios_hal::cpu::hlt();
    }
}

/// Init task — runs the shell after printing a startup message.
fn init_task() {
    minios_hal::serial_println!("init process (PID 1) running");
    loop {
        minios_hal::cpu::hlt();
    }
}

/// Smoke-tests the syscall dispatcher with uptime and getpid calls.
fn test_syscalls() {
    let uptime = minios_syscall::dispatch(minios_syscall::SYS_UPTIME, 0, 0, 0);
    minios_hal::serial_println!("Uptime via syscall: {} ticks", uptime);

    let pid = minios_syscall::dispatch(minios_syscall::SYS_GETPID, 0, 0, 0);
    minios_hal::serial_println!("PID via syscall: {}", pid);

    let msg = b"Hello from syscall write!\n";
    let written = minios_syscall::dispatch(
        minios_syscall::SYS_WRITE,
        1,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
    minios_hal::serial_println!("sys_write returned: {}", written);

    let unknown = minios_syscall::dispatch(999, 0, 0, 0);
    minios_hal::serial_println!("Unknown syscall returned: {} (expected -1)", unknown);

    minios_hal::serial_println!("Syscall subsystem OK");
}

/// Smoke-tests the IPC message queue: create, send, receive, destroy.
fn test_ipc() {
    use minios_common::id::Pid;

    minios_ipc::init();

    let mut mgr = minios_ipc::IPC_MANAGER.lock();

    let qid = mgr.create_queue_mut(8).expect("ipc: create queue failed");
    minios_hal::serial_println!("IPC: created queue {:?}", qid);

    let msg = minios_ipc::Message::new(Pid(0), 1, b"ping");
    mgr.send_message(qid, msg).expect("ipc: send failed");

    let received = mgr.receive_message(qid).expect("ipc: receive failed");
    minios_hal::serial_println!(
        "IPC: received {} bytes, type={}",
        received.data_len,
        received.msg_type
    );

    mgr.destroy_queue_mut(qid).expect("ipc: destroy failed");
    minios_hal::serial_println!("IPC subsystem OK");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    minios_hal::serial_println!("KERNEL PANIC: {}", info);
    minios_hal::cpu::hlt_loop();
}
