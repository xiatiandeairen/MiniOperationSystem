//! The `explain` command describes what a shell command does at the OS level.

use minios_hal::println;

pub fn cmd_explain(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: explain <command>");
        println!("Explains what happens inside the OS when a command runs.");
        return;
    }
    match args[0] {
        "ls" => explain_ls(),
        "cat" => explain_cat(),
        "ps" => explain_ps(),
        "meminfo" => explain_meminfo(),
        "trace" => explain_trace(),
        "spawn" => explain_spawn(),
        "help" => explain_help(),
        "echo" => explain_echo(),
        "write" => explain_write(),
        "mkdir" => explain_mkdir(),
        other => println!("No explanation available for '{}'.", other),
    }
}

fn explain_ls() {
    println!("=== How 'ls' works ===");
    println!("");
    println!("  1. Shell parses input -> command=\"ls\", args=[path]");
    println!("  2. VFS.open(path, READ)");
    println!("     -> Path resolution: split by '/', walk inode tree from root");
    println!("     -> Returns a FileDescriptor (integer handle)");
    println!("  3. VFS.readdir(fd)");
    println!("     -> RamFS iterates children of the directory inode");
    println!("     -> Returns list of (name, type, size)");
    println!("  4. For each entry: print to framebuffer console");
    println!("  5. VFS.close(fd) -> releases the file descriptor slot");
    println!("");
    println!("Key concepts:");
    println!("  - VFS: abstraction layer between commands and storage drivers");
    println!("  - Inode: unique ID for every file/directory (not the name!)");
    println!("  - File Descriptor: per-process handle to an open file");
}

fn explain_cat() {
    println!("=== How 'cat' works ===");
    println!("");
    println!("  1. VFS.open(path, READ) -> get file descriptor");
    println!("  2. Loop: VFS.read(fd, buffer, 512)");
    println!("     -> RamFS copies bytes from inode's data vector");
    println!("     -> Returns 0 when end-of-file reached");
    println!("  3. Print buffer contents to console");
    println!("  4. VFS.close(fd)");
    println!("");
    println!("For /proc/ files:");
    println!("  - ProcFS generates content on-the-fly (not stored on disk)");
    println!("  - /proc/meminfo reads live stats from the frame allocator");
    println!("  - /proc/uptime reads the timer tick counter");
}

fn explain_ps() {
    println!("=== How 'ps' works ===");
    println!("");
    println!("  1. Lock the process table (spinlock)");
    println!("  2. Iterate all 64 slots, skip empty ones");
    println!("  3. For each process: read PID, state, priority, CPU time");
    println!("  4. Print formatted table");
    println!("");
    println!("Process states: CREATED -> READY -> RUNNING -> BLOCKED -> TERMINATED");
    println!("The scheduler picks from READY tasks based on priority.");
}

fn explain_meminfo() {
    println!("=== How 'meminfo' works ===");
    println!("");
    println!("  1. Read frame allocator stats:");
    println!("     -> Bitmap tracks which 4KiB frames are used/free");
    println!("     -> Total frames = physical RAM / 4096");
    println!("  2. Read heap allocator stats:");
    println!("     -> Linked-list allocator tracks free blocks");
    println!("     -> Heap is a 1MiB region at virtual address 0x4444_4444_0000");
    println!("");
    println!("Memory hierarchy:");
    println!("  Physical frames (4KiB) -> Page tables -> Virtual addresses");
    println!("  Heap allocator manages fine-grained allocations within mapped pages");
}

fn explain_trace() {
    println!("=== How 'trace' works ===");
    println!("");
    println!("  The trace engine records timestamped spans for every OS operation.");
    println!("  Each span has: trace_id, span_id, parent, name, module, duration.");
    println!("");
    println!("  trace list   -> reads recent spans from a 4096-slot ring buffer");
    println!("  trace tree   -> reconstructs parent-child hierarchy from span_ids");
    println!("  trace follow -> clears buffer, runs a command, shows only new spans");
    println!("  trace export -> serializes spans as JSON to the serial port");
    println!("");
    println!("Ring buffer uses atomic write index. Old spans are overwritten.");
}

fn explain_spawn() {
    println!("=== How 'spawn' works ===");
    println!("");
    println!("  1. Allocate PID (atomic counter)");
    println!("  2. Allocate 16KiB kernel stack via heap");
    println!("  3. Create CpuContext: RSP=stack top, RIP=entry function");
    println!("  4. Insert into process table (array of 64 slots)");
    println!("  5. Add to scheduler's MLFQ queue");
    println!("");
    println!("The MLFQ scheduler has 4 priority queues:");
    println!("  Queue 0 [HIGH]:  time_slice = 2 ticks");
    println!("  Queue 1 [MED]:   time_slice = 4 ticks");
    println!("  Queue 2 [LOW]:   time_slice = 8 ticks");
    println!("  Queue 3 [IDLE]:  time_slice = 16 ticks");
}

fn explain_help() {
    println!("'help' simply iterates a static array of Command structs");
    println!("and prints each name + description. No OS primitives involved.");
}

fn explain_echo() {
    println!("'echo' writes arguments to the framebuffer console.");
    println!("Each character is drawn as 8x16 pixels using a bitmap font.");
    println!("The framebuffer is memory-mapped at the address provided by the bootloader.");
}

fn explain_write() {
    println!("=== How 'write' works ===");
    println!("");
    println!("  1. VFS.open(path, CREATE | WRITE)");
    println!("     -> If file doesn't exist: create new inode + add to parent");
    println!("  2. VFS.write(fd, data)");
    println!("     -> RamFS extends inode's data Vec, copies bytes in");
    println!("  3. VFS.close(fd)");
}

fn explain_mkdir() {
    println!("=== How 'mkdir' works ===");
    println!("");
    println!("  1. Resolve parent path -> get parent inode ID");
    println!("  2. Create new directory inode (type=Directory)");
    println!("  3. Add new inode ID to parent's children list");
    println!("  4. Insert inode into RamFS's BTreeMap<InodeId, Inode>");
}

pub fn cmd_tutorial(_args: &[&str]) {
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::GREEN);
    println!("+==========================================+");
    println!("|   Welcome to the MiniOS Tutorial!        |");
    println!("+==========================================+");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("");
    println!("Let's explore how an operating system works.");
    println!("Try these commands in order:");
    println!("");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    println!("  Step 1: help");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("    -> See all available commands");
    println!("");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    println!("  Step 2: ps");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("    -> See running processes (like Task Manager)");
    println!("");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    println!("  Step 3: ls /");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("    -> List the root filesystem");
    println!("");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    println!("  Step 4: explain ls");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("    -> Learn what happens INSIDE the OS when you run 'ls'");
    println!("");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    println!("  Step 5: trace follow ls /");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("    -> Watch the actual execution trace of 'ls'");
    println!("");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::YELLOW);
    println!("  Step 6: cat /proc/meminfo");
    minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    println!("    -> Read live memory statistics from the kernel");
    println!("");
    println!("Type 'explain <command>' for detailed explanations.");
    println!("Type 'tutorial' to see this guide again.");
}
