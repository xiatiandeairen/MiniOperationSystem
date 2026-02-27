//! Real-time system dashboard showing all key metrics at a glance.

use minios_hal::println;

/// Displays a comprehensive system dashboard.
pub fn cmd_dashboard(_args: &[&str]) {
    let mem = minios_memory::get_stats();
    let int_stats = minios_hal::interrupts::interrupt_stats();
    let sched = minios_scheduler::SCHEDULER.lock();
    let sched_stats = sched.stats();
    drop(sched);
    let procs = minios_process::manager::list_processes();

    use minios_common::traits::trace::Tracer;
    let trace_stats = minios_trace::TRACER.stats();

    println!("+------------------------------------------+");
    println!("|           MiniOS Dashboard               |");
    println!("+------------------------------------------+");
    println!(
        "| Uptime: {:>6}s  IRQs: timer={:<8} kb={} |",
        int_stats.timer_count / 100,
        int_stats.timer_count,
        int_stats.keyboard_count
    );
    println!("+------------------------------------------+");
    println!("| MEMORY                                   |");
    println!(
        "|   Frames: {:>5}/{:<5} ({:>3}% used)        |",
        mem.total_frames - mem.free_frames,
        mem.total_frames,
        (mem.total_frames - mem.free_frames) * 100 / mem.total_frames.max(1)
    );
    println!(
        "|   Heap:   {:>6} used / {:>6} free       |",
        mem.heap_used, mem.heap_free
    );
    println!("+------------------------------------------+");
    println!("| SCHEDULER                                |");
    println!(
        "|   Switches: {:<8}  Ticks: {:<10}   |",
        sched_stats.total_switches, sched_stats.total_ticks
    );
    println!(
        "|   Queues: H={} M={} L={} I={}              |",
        sched_stats.queue_lengths[0],
        sched_stats.queue_lengths[1],
        sched_stats.queue_lengths[2],
        sched_stats.queue_lengths[3]
    );
    println!("+------------------------------------------+");
    println!("| PROCESSES ({})                            |", procs.len());
    for p in &procs {
        let name = core::str::from_utf8(&p.name[..p.name_len]).unwrap_or("?");
        println!(
            "|   PID {:>2} {:8} {:>10} cpu={:<6}    |",
            p.pid, name, p.state, p.cpu_time_ticks
        );
    }
    println!("+------------------------------------------+");
    println!("| TRACE                                    |");
    println!(
        "|   Spans: {:<8}  Buffer: {}/{}       |",
        trace_stats.total_spans_written, trace_stats.buffer_used, trace_stats.buffer_capacity
    );
    println!("+------------------------------------------+");
}
