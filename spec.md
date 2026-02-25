# MiniOS — 完整技术规格说明书

> **项目名称**: MiniOperationSystem (MiniOS)
> **目标平台**: x86_64
> **开发语言**: Rust (`#![no_std]`) + 少量内联汇编
> **运行环境**: QEMU 虚拟机
> **核心特色**: 高度解耦的模块化设计 · 全链路 Trace 日志 · 可视化追踪链路

---

## 目录

1. [项目概览](#1-项目概览)
2. [设计哲学](#2-设计哲学)
3. [技术栈与工具链](#3-技术栈与工具链)
4. [系统架构总览](#4-系统架构总览)
5. [模块详细规格](#5-模块详细规格)
   - 5.1 [引导模块 (Boot)](#51-引导模块-boot)
   - 5.2 [硬件抽象层 (HAL)](#52-硬件抽象层-hal)
   - 5.3 [内存管理 (Memory)](#53-内存管理-memory)
   - 5.4 [中断与异常处理 (Interrupt)](#54-中断与异常处理-interrupt)
   - 5.5 [进程与线程管理 (Process)](#55-进程与线程管理-process)
   - 5.6 [调度器 (Scheduler)](#56-调度器-scheduler)
   - 5.7 [进程间通信 (IPC)](#57-进程间通信-ipc)
   - 5.8 [文件系统 (FileSystem)](#58-文件系统-filesystem)
   - 5.9 [设备驱动 (Driver)](#59-设备驱动-driver)
   - 5.10 [系统调用接口 (Syscall)](#510-系统调用接口-syscall)
   - 5.11 [Shell 交互终端](#511-shell-交互终端)
6. [核心特色：Trace 追踪系统](#6-核心特色trace-追踪系统)
   - 6.1 [Trace 引擎](#61-trace-引擎)
   - 6.2 [Trace 日志格式](#62-trace-日志格式)
   - 6.3 [内核内 Trace 查看器](#63-内核内-trace-查看器)
   - 6.4 [宿主机可视化工具](#64-宿主机可视化工具)
7. [解耦设计详述](#7-解耦设计详述)
8. [目录结构](#8-目录结构)
9. [构建系统](#9-构建系统)
10. [测试策略](#10-测试策略)
11. [开发阶段规划](#11-开发阶段规划)
12. [关键接口定义](#12-关键接口定义)
13. [错误处理策略](#13-错误处理策略)
14. [性能约束与目标](#14-性能约束与目标)

---

## 1. 项目概览

### 1.1 项目目标

构建一个可引导、可运行的微型操作系统内核，具备操作系统的核心功能，同时引入三大创新特色：

| 特色 | 说明 |
|------|------|
| **解耦设计** | 每个子系统通过 Rust trait 定义接口边界，模块间零直接依赖，支持替换任何子系统实现 |
| **全链路 Trace** | 从引导到用户态 Shell 命令执行，每一个操作都生成结构化 trace span，形成完整调用链 |
| **可视化追踪** | 内核内置文本模式 trace 查看器 + 宿主机 Web 可视化工具，实时展示 span 瀑布图 |

### 1.2 功能范围

| 功能领域 | 实现范围 | 不包含 |
|---------|---------|--------|
| 引导 | BIOS/UEFI 通过 bootloader crate 引导进入 64 位长模式 | 自写引导扇区 |
| 内存 | 物理帧分配器 + 4 级页表虚拟内存 + 内核堆分配器 | Swap / 磁盘换页 |
| 进程 | 内核态任务（线程） + 基本进程抽象 + 上下文切换 | 完整用户态隔离 |
| 调度 | 多级反馈队列 (MLFQ) 调度器 | SMP 多核调度 |
| 中断 | IDT + PIC/APIC 中断处理 + 键盘/时钟中断 | IOAPIC 完整支持 |
| 文件系统 | VFS 抽象层 + 内存文件系统 (RamFS) | 磁盘文件系统 |
| 驱动 | VGA 文本模式 + 串口 + PS/2 键盘 | 网络/USB/GPU |
| 系统调用 | 基础系统调用（read/write/open/close/fork/exit/exec 等） | POSIX 完整兼容 |
| Shell | 内置 Shell + 基础命令 + trace 查看命令 | 图形界面 |
| IPC | 消息队列 + 共享内存（简化） | 信号量/Socket |

### 1.3 非功能性需求

- **可测试性**: 每个模块可独立进行单元测试（`#[cfg(test)]`）
- **可追踪性**: 100% 的跨模块调用产生 trace span
- **可观测性**: 通过串口实时输出 trace 数据到宿主机
- **构建简便**: 单命令 `cargo make run` 即可编译并在 QEMU 中启动
- **文档完备**: 每个公开接口附带 rustdoc 文档

---

## 2. 设计哲学

### 2.1 核心原则

```
┌─────────────────────────────────────────────────────┐
│                   设计原则金字塔                       │
│                                                     │
│                    ┌───────┐                         │
│                    │ 可追踪 │  ← Trace 贯穿一切       │
│                   ┌┴───────┴┐                       │
│                   │  可观测  │  ← 系统行为透明可见     │
│                  ┌┴─────────┴┐                      │
│                  │   可替换   │  ← 任何模块可热替换    │
│                 ┌┴───────────┴┐                     │
│                 │    可测试    │  ← 独立单元测试       │
│                ┌┴─────────────┴┐                    │
│                │     解耦      │  ← 零直接依赖       │
│                └───────────────┘                    │
└─────────────────────────────────────────────────────┘
```

### 2.2 解耦策略

**接口驱动设计 (Interface-Driven Design)**：

- 每个子系统暴露一个 Rust trait 作为公共接口
- 子系统之间只依赖 trait，不依赖具体实现
- 通过一个全局的 `KernelServices` 注册表管理所有子系统实例
- 支持在编译期（通过 feature flag）或初始化期切换实现

**依赖方向规则**：

```
Shell → Syscall → Process/FS/Memory → HAL → Hardware
                         ↑
                    Trace（横切所有层）
```

- 上层可以依赖下层 trait
- 下层不可以依赖上层
- Trace 系统作为横切关注点，通过宏注入，不增加模块间耦合

### 2.3 Trace-First 开发

每写一个功能，同步编写对应的 trace instrumentation。Trace 不是事后添加，而是功能的一等公民。

---

## 3. 技术栈与工具链

### 3.1 语言与核心依赖

| 组件 | 选型 | 版本/说明 |
|------|------|----------|
| 语言 | Rust | nightly (需要 `asm`, `alloc`, `naked_fn` 等不稳定特性) |
| 引导 | `bootloader` crate | v0.11+ (处理 BIOS/UEFI 引导和长模式切换) |
| CPU 操作 | `x86_64` crate | 页表操作、特权级、MSR、端口 I/O 等 |
| 同步原语 | `spin` crate | 自旋锁（内核中不能用 std Mutex） |
| 串口 | `uart_16550` crate | 串口驱动（trace 输出通道） |
| VGA | `volatile` crate | VGA 缓冲区的 volatile 读写 |
| 位操作 | `bitflags` crate | 标志位定义 |
| 日志格式 | 自研 | 二进制 trace 格式 + JSON 导出 |

### 3.2 构建与运行

| 工具 | 用途 |
|------|------|
| `cargo` | Rust 构建系统 |
| `cargo-make` | 任务编排（编译 → 创建磁盘镜像 → 启动 QEMU） |
| `rust-src` | 交叉编译标准库源码 |
| `llvm-tools-preview` | 链接器和二进制工具 |
| `qemu-system-x86_64` | 虚拟机运行和调试 |
| `gdb` / `lldb` | 远程调试（可选） |

### 3.3 宿主机可视化工具

| 工具 | 用途 |
|------|------|
| HTML + CSS + JavaScript | 浏览器端 trace 可视化 |
| 无额外框架 | 纯 Vanilla JS，无需 Node.js |
| WebSocket (可选) | 实时接收串口 trace 数据 |

### 3.4 编译目标

```json
{
  "llvm-target": "x86_64-unknown-none",
  "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "os": "none",
  "features": "-mmx,-sse,+soft-float",
  "linker-flavor": "ld.lld",
  "linker": "rust-lld",
  "panic-strategy": "abort",
  "disable-redzone": true
}
```

---

## 4. 系统架构总览

### 4.1 分层架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户交互层 (Layer 5)                       │
│  ┌─────────┐  ┌──────────┐  ┌──────────────┐                   │
│  │  Shell   │  │ 用户命令  │  │  Trace 查看器 │                   │
│  └────┬─────┘  └────┬─────┘  └──────┬───────┘                   │
├───────┼─────────────┼───────────────┼───────────────────────────┤
│       │        系统调用层 (Layer 4)   │                           │
│  ┌────▼─────────────▼───────────────▼───────┐                   │
│  │          Syscall Dispatcher               │                   │
│  │    (read/write/open/fork/exec/exit...)    │                   │
│  └────┬─────────────┬───────────────┬───────┘                   │
├───────┼─────────────┼───────────────┼───────────────────────────┤
│       │      内核服务层 (Layer 3)     │                           │
│  ┌────▼────┐  ┌─────▼─────┐  ┌─────▼─────┐  ┌──────────┐      │
│  │ Process │  │    VFS    │  │    IPC    │  │ Scheduler│      │
│  │ Manager │  │  + RamFS  │  │  MsgQueue │  │  (MLFQ)  │      │
│  └────┬────┘  └─────┬─────┘  └─────┬─────┘  └────┬─────┘      │
├───────┼─────────────┼───────────────┼─────────────┼─────────────┤
│       │     核心内核层 (Layer 2)      │             │             │
│  ┌────▼────────┐  ┌─▼──────────┐  ┌─▼─────────────▼──┐         │
│  │   Memory    │  │ Interrupt  │  │   Task Switch    │         │
│  │  Manager    │  │  Handler   │  │  (Context Save)  │         │
│  │ Phys+Virt   │  │  IDT+PIC   │  │                  │         │
│  └────┬────────┘  └─┬──────────┘  └──────────────────┘         │
├───────┼─────────────┼───────────────────────────────────────────┤
│       │     硬件抽象层 (Layer 1)                                  │
│  ┌────▼────┐  ┌─────▼─────┐  ┌──────────┐  ┌──────────┐       │
│  │  Port   │  │    CPU    │  │  Serial  │  │   VGA    │       │
│  │  I/O    │  │ (Regs/CR) │  │  UART    │  │ TextBuf  │       │
│  └────┬────┘  └─────┬─────┘  └────┬─────┘  └────┬─────┘       │
├───────┼─────────────┼──────────────┼─────────────┼──────────────┤
│       ▼             ▼              ▼             ▼              │
│                        硬件 (Layer 0)                            │
│         CPU · RAM · 串口 · 键盘 · VGA 显示器                     │
└─────────────────────────────────────────────────────────────────┘

                    ╔═══════════════════╗
                    ║   Trace Engine    ║  ← 横切所有层
                    ║  (Cross-Cutting)  ║
                    ╚═══════════════════╝
```

### 4.2 模块依赖关系图

```
                         ┌────────┐
                         │ shell  │
                         └───┬────┘
                             │
                         ┌───▼────┐
                    ┌────│syscall │────┐
                    │    └───┬────┘    │
                    │        │         │
               ┌────▼──┐ ┌──▼───┐ ┌──▼────┐
               │process│ │  fs  │ │  ipc  │
               └───┬───┘ └──┬───┘ └──┬────┘
                   │        │        │
              ┌────▼────────▼────────▼────┐
              │      kernel_core          │
              │ (memory + interrupt +     │
              │  task_switch)             │
              └──────────┬────────────────┘
                         │
                    ┌────▼────┐
                    │   hal   │
                    └────┬────┘
                         │
                    ┌────▼────┐
                    │hardware │
                    └─────────┘

     ┌─────────────────────────────────┐
     │          trace (横切)            │
     │  通过 #[trace] 宏注入到所有模块   │
     └─────────────────────────────────┘
```

### 4.3 KernelServices 注册表

所有子系统通过一个全局注册表进行管理，消除模块间直接依赖：

```rust
pub struct KernelServices {
    pub memory:    &'static dyn MemoryManager,
    pub process:   &'static dyn ProcessManager,
    pub scheduler: &'static dyn Scheduler,
    pub fs:        &'static dyn FileSystem,
    pub interrupt: &'static dyn InterruptController,
    pub ipc:       &'static dyn IpcManager,
    pub tracer:    &'static dyn Tracer,
}

static KERNEL: Once<KernelServices> = Once::new();

pub fn kernel() -> &'static KernelServices {
    KERNEL.get().expect("Kernel not initialized")
}
```

---

## 5. 模块详细规格

### 5.1 引导模块 (Boot)

#### 职责

- 使用 `bootloader` crate 从 BIOS/UEFI 引导
- 接收引导信息（内存映射表、帧缓冲区等）
- 初始化各子系统并构建 `KernelServices`
- 启动第一个用户态任务（Shell）

#### 引导流程

```
BIOS/UEFI
  │
  ▼
bootloader crate (设置长模式、页表、栈)
  │
  ▼
_start() / kernel_main(boot_info)
  │
  ├─ 1. 初始化 HAL (串口、VGA)
  ├─ 2. 初始化 Trace Engine (最早初始化，后续步骤都有 trace)
  ├─ 3. 初始化内存管理 (物理帧分配器 + 页表 + 堆)
  ├─ 4. 初始化中断 (IDT + PIC + 开启中断)
  ├─ 5. 初始化进程管理 + 调度器
  ├─ 6. 初始化文件系统 (VFS + RamFS 挂载)
  ├─ 7. 初始化 IPC
  ├─ 8. 初始化系统调用表
  ├─ 9. 注册 KernelServices
  ├─ 10. 创建 init 进程 (PID 1)
  └─ 11. 启动调度器 (不再返回)
```

#### 初始化 Trace 示例

```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    let _span = trace_span!("kernel_boot", module = "boot");

    hal::init();
    trace_event!("hal_initialized");

    tracer::init();
    trace_event!("tracer_initialized");

    let mem = memory::init(&boot_info.memory_map);
    trace_event!("memory_initialized", total_frames = mem.total_frames());

    // ... 其余初始化，每一步都有 trace
}
```

---

### 5.2 硬件抽象层 (HAL)

#### 职责

- 封装所有直接硬件操作（端口 I/O、CPU 寄存器、中断控制器）
- 提供平台无关的接口 trait
- 为上层屏蔽 x86_64 特有细节

#### 子模块

| 子模块 | 功能 | 关键类型/函数 |
|--------|------|--------------|
| `hal::port` | 端口 I/O 读写 | `Port<T>`, `PortReadOnly<T>`, `PortWriteOnly<T>` |
| `hal::cpu` | CPU 控制（中断开关、halt、特权指令） | `enable_interrupts()`, `disable_interrupts()`, `hlt()` |
| `hal::serial` | 串口初始化与输出 | `SerialPort`, `init()`, `write_byte()`, `write_str()` |
| `hal::vga` | VGA 文本模式缓冲区 | `VgaBuffer`, `write_char()`, `set_color()`, `scroll()` |
| `hal::pic` | PIC 8259 中断控制器 | `ChainedPics`, `init()`, `end_of_interrupt()` |
| `hal::gdt` | GDT / TSS 设置 | `init()`, `Gdt`, `TaskStateSegment` |

#### HAL Trait 定义

```rust
pub trait HalSerial: Send + Sync {
    fn write_byte(&self, byte: u8);
    fn write_bytes(&self, bytes: &[u8]);
    fn read_byte(&self) -> Option<u8>;
}

pub trait HalDisplay: Send + Sync {
    fn write_char(&self, row: usize, col: usize, ch: u8, color: ColorCode);
    fn scroll_up(&self);
    fn clear(&self);
    fn dimensions(&self) -> (usize, usize); // (rows, cols)
}

pub trait HalInterruptController: Send + Sync {
    fn init(&self);
    fn enable_irq(&self, irq: u8);
    fn disable_irq(&self, irq: u8);
    fn end_of_interrupt(&self, irq: u8);
}
```

---

### 5.3 内存管理 (Memory)

#### 职责

- 管理物理内存帧的分配与释放
- 管理虚拟地址空间（4 级页表映射）
- 提供内核堆分配器（支持 `alloc::*`）

#### 5.3.1 物理帧分配器

**算法**: Bitmap 分配器（简单可靠，适合 trace 记录）

```rust
pub trait FrameAllocator: Send + Sync {
    fn allocate_frame(&self) -> Option<PhysFrame>;
    fn deallocate_frame(&self, frame: PhysFrame);
    fn free_frames(&self) -> usize;
    fn total_frames(&self) -> usize;
}
```

**实现细节**：

| 属性 | 说明 |
|------|------|
| 帧大小 | 4 KiB (标准 x86_64 页大小) |
| 管理结构 | Bitmap，每 bit 对应一个物理帧 |
| 初始化 | 从 bootloader 提供的 MemoryMap 构建 |
| 线程安全 | 通过 SpinLock 保护 |
| Trace | 每次 alloc/dealloc 记录帧号和调用源 |

**Trace 输出示例**：
```
[TRACE] span_id=42 parent=10 module=memory op=frame_alloc frame=0x1A3 caller=process::create duration=120ns
[TRACE] span_id=43 parent=10 module=memory op=frame_dealloc frame=0x1A3 caller=process::exit duration=45ns
```

#### 5.3.2 虚拟内存管理

**页表结构**: 4 级页表 (PML4 → PDPT → PD → PT)

```rust
pub trait VirtualMemoryManager: Send + Sync {
    fn map_page(
        &self,
        page: Page,
        frame: PhysFrame,
        flags: PageFlags,
        allocator: &dyn FrameAllocator,
    ) -> Result<(), MapError>;

    fn unmap_page(&self, page: Page) -> Result<PhysFrame, UnmapError>;

    fn translate_addr(&self, virt: VirtAddr) -> Option<PhysAddr>;

    fn create_address_space(&self) -> Result<AddressSpace, MemoryError>;
    fn switch_address_space(&self, space: &AddressSpace);
}
```

**地址空间布局**:

```
虚拟地址空间 (48-bit canonical)
┌──────────────────────────────────────┐ 0xFFFF_FFFF_FFFF_FFFF
│            内核空间                    │
│  ┌────────────────────────────┐      │ 0xFFFF_8000_0000_0000
│  │ 内核代码 + 数据              │      │
│  │ 内核堆                      │      │
│  │ 物理内存直接映射              │      │
│  │ Trace Ring Buffer           │      │
│  └────────────────────────────┘      │
├──────────────────────────────────────┤ 0x0000_8000_0000_0000
│            (非规范空间 - 不可用)       │
├──────────────────────────────────────┤ 0x0000_7FFF_FFFF_FFFF
│            用户空间                    │
│  ┌────────────────────────────┐      │
│  │ 用户栈 (向下增长)            │      │ 0x0000_7FFF_FFFF_F000
│  │          ...                │      │
│  │ 用户堆 (向上增长)            │      │
│  │ 用户数据段                   │      │
│  │ 用户代码段                   │      │ 0x0000_0000_0040_0000
│  └────────────────────────────┘      │
│  (空白/保护区)                        │ 0x0000_0000_0000_1000
│  NULL 保护页                          │ 0x0000_0000_0000_0000
└──────────────────────────────────────┘
```

#### 5.3.3 内核堆分配器

**算法**: Linked List Allocator（初期简单）→ 后续可替换为 Slab Allocator

```rust
pub trait HeapAllocator: Send + Sync {
    fn allocate(&self, layout: Layout) -> *mut u8;
    fn deallocate(&self, ptr: *mut u8, layout: Layout);
    fn used_bytes(&self) -> usize;
    fn free_bytes(&self) -> usize;
}
```

**配置**:

| 参数 | 值 |
|------|-----|
| 堆起始地址 | `0xFFFF_8888_0000_0000` |
| 初始堆大小 | 1 MiB |
| 最大堆大小 | 16 MiB（按需扩展） |
| 对齐 | 最小 8 字节对齐 |

---

### 5.4 中断与异常处理 (Interrupt)

#### 职责

- 设置并管理中断描述符表 (IDT)
- 处理 CPU 异常（Page Fault、Double Fault、GP Fault 等）
- 处理硬件中断（时钟、键盘）
- 提供中断注册接口

#### IDT 配置

| 向量号 | 类型 | 处理内容 |
|--------|------|---------|
| 0 | Division Error | panic + trace |
| 6 | Invalid Opcode | panic + trace |
| 8 | Double Fault | panic + trace (使用 IST) |
| 13 | General Protection Fault | panic + trace |
| 14 | Page Fault | 页面错误处理/按需分配 + trace |
| 32 | Timer (IRQ0) | 调度器 tick + trace |
| 33 | Keyboard (IRQ1) | 键盘输入处理 + trace |
| 128 (0x80) | 系统调用 | syscall 分发 + trace |

#### Interrupt Trait

```rust
pub trait InterruptHandler: Send + Sync {
    fn handle(&self, vector: u8, frame: &InterruptFrame) -> InterruptResult;
}

pub trait InterruptManager: Send + Sync {
    fn register_handler(&self, vector: u8, handler: Box<dyn InterruptHandler>);
    fn enable(&self);
    fn disable(&self);
}
```

#### 时钟中断处理流程

```
Timer IRQ (每 10ms)
  │
  ├─ trace_begin("timer_tick")
  ├─ 更新系统时钟计数
  ├─ 调用 scheduler.tick()
  │   ├─ 当前任务时间片 -1
  │   └─ 若时间片耗尽 → 标记需要调度
  ├─ 发送 EOI
  ├─ trace_end()
  └─ 若需要调度 → 触发上下文切换
```

---

### 5.5 进程与线程管理 (Process)

#### 职责

- 创建、销毁进程/线程
- 管理进程控制块 (PCB)
- 执行上下文切换
- 管理 PID 分配

#### 进程控制块 (PCB)

```rust
pub struct Process {
    pub pid: Pid,
    pub name: ArrayString<32>,
    pub state: ProcessState,
    pub parent_pid: Option<Pid>,
    pub context: CpuContext,
    pub address_space: AddressSpace,
    pub kernel_stack: VirtAddr,
    pub kernel_stack_size: usize,
    pub file_descriptors: FileDescriptorTable,
    pub exit_code: Option<i32>,
    pub priority: Priority,
    pub cpu_time: u64,           // TSC cycles consumed
    pub created_at: u64,         // TSC at creation
    pub trace_context: TraceContext, // 当前 trace 上下文
}
```

#### 进程状态机

```
                 ┌──────────┐
      fork()     │          │  exec()
     ┌──────────►│ Created  ├──────────┐
     │           │          │          │
     │           └────┬─────┘          │
     │                │ schedule()     │
     │           ┌────▼─────┐          │
     │     ┌─────│  Ready   │◄────┐    │
     │     │     └────┬─────┘     │    │
     │     │          │ dispatch  │    │
     │     │     ┌────▼─────┐     │    │
     │     │     │ Running  │─────┘    │
     │     │     └────┬─────┘ preempt/ │
     │     │          │ yield          │
     │     │     ┌────▼─────┐          │
     │     └────►│ Blocked  │          │
     │    I/O完成 └────┬─────┘          │
     │                │ wakeup         │
     │           ┌────▼─────┐          │
     │           │  Ready   │          │
     │           └──────────┘          │
     │                                 │
     │           ┌──────────┐          │
     └──────────►│Terminated│◄─────────┘
                 │          │  exit()
                 └──────────┘
```

#### ProcessManager Trait

```rust
pub trait ProcessManager: Send + Sync {
    fn create_process(&self, name: &str, entry: fn()) -> Result<Pid, ProcessError>;
    fn exit_process(&self, pid: Pid, code: i32) -> Result<(), ProcessError>;
    fn kill_process(&self, pid: Pid) -> Result<(), ProcessError>;
    fn get_process(&self, pid: Pid) -> Option<&Process>;
    fn current_pid(&self) -> Pid;
    fn list_processes(&self) -> Vec<ProcessInfo>;
    fn fork(&self) -> Result<Pid, ProcessError>;
    fn waitpid(&self, pid: Pid) -> Result<i32, ProcessError>;
}
```

#### 上下文切换

```rust
#[repr(C)]
pub struct CpuContext {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub r8: u64,  pub r9: u64,  pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cr3: u64,    // 页表基地址
    pub fs_base: u64,
    pub gs_base: u64,
}

// 上下文切换 (汇编实现)
extern "C" {
    fn switch_context(old: *mut CpuContext, new: *const CpuContext);
}
```

**上下文切换 Trace**：

```
[TRACE] span=switch_context from_pid=3 to_pid=5 from_state=Running to_state=Ready reason=timeslice_expired duration=850ns
```

---

### 5.6 调度器 (Scheduler)

#### 算法：多级反馈队列 (MLFQ)

```
优先级 0 (最高): ┌─────────────────────┐  时间片: 2 ticks
                 │ 交互式/新创建的任务   │
                 └─────────────────────┘
优先级 1:        ┌─────────────────────┐  时间片: 4 ticks
                 │ 中等优先级任务        │
                 └─────────────────────┘
优先级 2:        ┌─────────────────────┐  时间片: 8 ticks
                 │ 计算密集型任务        │
                 └─────────────────────┘
优先级 3 (最低): ┌─────────────────────┐  时间片: 16 ticks
                 │ 后台/空闲任务        │
                 └─────────────────────┘
```

#### 调度规则

1. 新任务进入最高优先级队列
2. 任务用完时间片后降级到下一级队列
3. 任务主动让出 CPU（I/O 等待）后保持或提升优先级
4. 定期 boost：每 N 个 tick 将所有任务重置到最高优先级（防止饥饿）

#### Scheduler Trait

```rust
pub trait Scheduler: Send + Sync {
    fn add_task(&self, pid: Pid, priority: Priority);
    fn remove_task(&self, pid: Pid);
    fn tick(&self) -> ScheduleDecision;
    fn next_task(&self) -> Option<Pid>;
    fn yield_current(&self);
    fn block_current(&self, reason: BlockReason);
    fn unblock(&self, pid: Pid);
    fn set_priority(&self, pid: Pid, priority: Priority);
    fn stats(&self) -> SchedulerStats;
}

pub enum ScheduleDecision {
    Continue,              // 当前任务继续
    Switch(Pid),           // 切换到指定任务
    Idle,                  // 无就绪任务，进入 idle
}

pub struct SchedulerStats {
    pub total_switches: u64,
    pub total_ticks: u64,
    pub queue_lengths: [usize; 4],
    pub idle_ticks: u64,
}
```

---

### 5.7 进程间通信 (IPC)

#### 机制一：消息队列

```rust
pub struct Message {
    pub sender: Pid,
    pub msg_type: u32,
    pub payload: [u8; 256],
    pub payload_len: usize,
    pub timestamp: u64,
    pub trace_context: TraceContext, // 关键：trace 上下文跨进程传播
}

pub trait MessageQueue: Send + Sync {
    fn create(&self, name: &str, capacity: usize) -> Result<QueueId, IpcError>;
    fn send(&self, queue: QueueId, msg: &Message) -> Result<(), IpcError>;
    fn receive(&self, queue: QueueId, timeout: Option<u64>) -> Result<Message, IpcError>;
    fn destroy(&self, queue: QueueId) -> Result<(), IpcError>;
}
```

#### 机制二：共享内存

```rust
pub trait SharedMemory: Send + Sync {
    fn create(&self, name: &str, size: usize) -> Result<ShmId, IpcError>;
    fn attach(&self, shm: ShmId, pid: Pid) -> Result<VirtAddr, IpcError>;
    fn detach(&self, shm: ShmId, pid: Pid) -> Result<(), IpcError>;
    fn destroy(&self, shm: ShmId) -> Result<(), IpcError>;
}
```

#### IPC Trace 传播

**关键设计**: 消息中携带 `TraceContext`，使得跨进程调用链可追踪。

```
Process A                    Process B
    │                            │
    ├─ span: "send_request"      │
    │   trace_id: 0xABC          │
    │   span_id: 10              │
    │       │                    │
    │   msg.trace_context =      │
    │     { trace_id: 0xABC,     │
    │       span_id: 10 }        │
    │       │                    │
    │   ══► MsgQueue ══►         │
    │                    ├─ span: "handle_request"
    │                    │   trace_id: 0xABC  (同一条 trace!)
    │                    │   parent_span: 10  (关联到发送方)
    │                    │   span_id: 20
```

---

### 5.8 文件系统 (FileSystem)

#### 5.8.1 VFS 虚拟文件系统层

```rust
pub trait FileSystem: Send + Sync {
    fn mount(&self, path: &str, fs: Box<dyn FileSystemDriver>) -> Result<(), FsError>;
    fn unmount(&self, path: &str) -> Result<(), FsError>;

    fn open(&self, path: &str, flags: OpenFlags) -> Result<FileDescriptor, FsError>;
    fn close(&self, fd: FileDescriptor) -> Result<(), FsError>;
    fn read(&self, fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&self, fd: FileDescriptor, buf: &[u8]) -> Result<usize, FsError>;
    fn seek(&self, fd: FileDescriptor, offset: i64, whence: SeekWhence) -> Result<u64, FsError>;

    fn mkdir(&self, path: &str) -> Result<(), FsError>;
    fn rmdir(&self, path: &str) -> Result<(), FsError>;
    fn unlink(&self, path: &str) -> Result<(), FsError>;
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, FsError>;
    fn stat(&self, path: &str) -> Result<FileStat, FsError>;
}
```

#### 5.8.2 文件系统驱动 Trait

```rust
pub trait FileSystemDriver: Send + Sync {
    fn name(&self) -> &str;
    fn create_file(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn create_dir(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn read_data(&self, inode: InodeId, offset: usize, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write_data(&self, inode: InodeId, offset: usize, buf: &[u8]) -> Result<usize, FsError>;
    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn remove(&self, parent: InodeId, name: &str) -> Result<(), FsError>;
    fn list(&self, inode: InodeId) -> Result<Vec<DirEntry>, FsError>;
    fn stat(&self, inode: InodeId) -> Result<FileStat, FsError>;
}
```

#### 5.8.3 RamFS 实现

**数据结构**:

```rust
struct RamFsInode {
    id: InodeId,
    inode_type: InodeType,  // File | Directory
    name: ArrayString<255>,
    data: Vec<u8>,          // 文件内容（仅 File）
    children: Vec<InodeId>, // 子节点（仅 Directory）
    parent: Option<InodeId>,
    size: usize,
    created_at: u64,
    modified_at: u64,
    permissions: u16,
}
```

**初始文件系统结构**:

```
/
├── dev/
│   ├── null      (空设备)
│   ├── zero      (零设备)
│   ├── serial0   (串口设备)
│   └── console   (VGA 控制台)
├── proc/
│   ├── self/     (当前进程信息)
│   └── [pid]/    (各进程信息)
├── tmp/          (临时文件)
├── trace/
│   ├── current   (当前 trace 数据，只读)
│   └── config    (trace 配置)
└── etc/
    └── motd      (登录信息)
```

#### 5.8.4 特殊文件系统: ProcFS

通过 `/proc/` 暴露内核状态：

| 路径 | 内容 |
|------|------|
| `/proc/meminfo` | 内存使用统计 |
| `/proc/cpuinfo` | CPU 信息 |
| `/proc/uptime` | 系统运行时间 |
| `/proc/[pid]/status` | 进程状态 |
| `/proc/[pid]/trace` | 进程 trace 上下文 |

#### 5.8.5 特殊文件系统: TraceFS

通过 `/trace/` 访问 trace 数据：

| 路径 | 内容 |
|------|------|
| `/trace/current` | 当前 ring buffer 中的 trace 数据（JSON 格式） |
| `/trace/config` | trace 配置（级别、过滤器等） |
| `/trace/stats` | trace 统计信息 |

---

### 5.9 设备驱动 (Driver)

#### 驱动模型

```rust
pub trait DeviceDriver: Send + Sync {
    fn name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
    fn init(&self) -> Result<(), DriverError>;
    fn read(&self, buf: &mut [u8]) -> Result<usize, DriverError>;
    fn write(&self, buf: &[u8]) -> Result<usize, DriverError>;
    fn ioctl(&self, cmd: u32, arg: usize) -> Result<usize, DriverError>;
}

pub enum DeviceType {
    CharDevice,   // 字符设备 (串口, 键盘, VGA)
    BlockDevice,  // 块设备 (未来: 磁盘)
}
```

#### 驱动清单

| 驱动 | 类型 | 功能 |
|------|------|------|
| `VgaTextDriver` | CharDevice | 80x25 文本模式显示，支持颜色，自动滚屏 |
| `SerialDriver` | CharDevice | UART 16550 串口，用于 trace 输出和调试 |
| `KeyboardDriver` | CharDevice | PS/2 键盘，扫描码转换，支持 Shift/Ctrl |
| `NullDriver` | CharDevice | `/dev/null` — 丢弃所有写入，读取返回 EOF |
| `ZeroDriver` | CharDevice | `/dev/zero` — 读取返回 0，写入丢弃 |

#### 键盘驱动详细设计

```rust
pub struct KeyboardEvent {
    pub scancode: u8,
    pub key: Option<KeyCode>,
    pub modifiers: Modifiers,
    pub pressed: bool, // true = press, false = release
}

pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub caps_lock: bool,
}

pub trait KeyboardListener: Send + Sync {
    fn on_key_event(&self, event: KeyboardEvent);
}
```

---

### 5.10 系统调用接口 (Syscall)

#### 系统调用表

| 编号 | 名称 | 签名 | 说明 |
|------|------|------|------|
| 0 | `sys_read` | `(fd, buf, len) -> isize` | 从文件描述符读取 |
| 1 | `sys_write` | `(fd, buf, len) -> isize` | 写入文件描述符 |
| 2 | `sys_open` | `(path, flags) -> isize` | 打开文件 |
| 3 | `sys_close` | `(fd) -> isize` | 关闭文件描述符 |
| 4 | `sys_mkdir` | `(path) -> isize` | 创建目录 |
| 5 | `sys_readdir` | `(fd, buf, count) -> isize` | 读取目录项 |
| 6 | `sys_stat` | `(path, stat_buf) -> isize` | 获取文件状态 |
| 7 | `sys_unlink` | `(path) -> isize` | 删除文件 |
| 10 | `sys_fork` | `() -> isize` | 创建子进程 |
| 11 | `sys_exec` | `(path, argv) -> isize` | 执行程序 |
| 12 | `sys_exit` | `(code) -> !` | 退出进程 |
| 13 | `sys_waitpid` | `(pid) -> isize` | 等待子进程结束 |
| 14 | `sys_getpid` | `() -> isize` | 获取当前 PID |
| 15 | `sys_yield` | `() -> isize` | 让出 CPU |
| 16 | `sys_sleep` | `(ms) -> isize` | 睡眠 |
| 20 | `sys_mmap` | `(addr, len, prot, flags) -> isize` | 内存映射 |
| 21 | `sys_munmap` | `(addr, len) -> isize` | 取消内存映射 |
| 30 | `sys_send` | `(queue, msg) -> isize` | IPC 发送消息 |
| 31 | `sys_recv` | `(queue, buf, timeout) -> isize` | IPC 接收消息 |
| 40 | `sys_trace_dump` | `(buf, len) -> isize` | 导出 trace 数据 |
| 41 | `sys_trace_config` | `(key, value) -> isize` | 配置 trace |
| 50 | `sys_uptime` | `() -> isize` | 获取系统运行时间 |
| 51 | `sys_meminfo` | `(buf) -> isize` | 获取内存信息 |

#### Syscall 分发器

```rust
pub fn syscall_dispatch(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) -> i64 {
    let _span = trace_span!(
        "syscall",
        module = "syscall",
        syscall_num = num,
        pid = current_pid(),
    );

    match num {
        0 => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        1 => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        // ...
        _ => -1, // ENOSYS
    }
}
```

---

### 5.11 Shell 交互终端

#### 功能列表

| 命令 | 用法 | 说明 |
|------|------|------|
| `help` | `help [command]` | 显示帮助信息 |
| `echo` | `echo <text>` | 输出文本 |
| `clear` | `clear` | 清屏 |
| `ls` | `ls [path]` | 列出目录内容 |
| `cd` | `cd <path>` | 切换目录 |
| `pwd` | `pwd` | 显示当前路径 |
| `cat` | `cat <file>` | 显示文件内容 |
| `mkdir` | `mkdir <path>` | 创建目录 |
| `touch` | `touch <file>` | 创建文件 |
| `rm` | `rm <path>` | 删除文件 |
| `write` | `write <file> <content>` | 写入文件 |
| `ps` | `ps` | 列出进程 |
| `kill` | `kill <pid>` | 终止进程 |
| `meminfo` | `meminfo` | 显示内存信息 |
| `uptime` | `uptime` | 显示运行时间 |
| `trace` | `trace [subcmd]` | **Trace 子系统命令** |
| `trace list` | `trace list [-n N]` | 显示最近 N 条 trace span |
| `trace tree` | `trace tree [trace_id]` | 树形显示某条 trace 链路 |
| `trace stats` | `trace stats` | 显示 trace 统计信息 |
| `trace filter` | `trace filter <module>` | 按模块过滤 trace |
| `trace export` | `trace export` | 通过串口导出 trace 数据 (JSON) |
| `trace clear` | `trace clear` | 清空 trace buffer |
| `trace live` | `trace live` | 实时 trace 输出模式 |

#### Shell 输入处理流程

```
键盘中断 → KeyboardDriver → InputBuffer → Shell::process_input()
                                              │
                                              ├─ 回车 → parse_command() → execute_command()
                                              ├─ 退格 → 删除字符
                                              ├─ Ctrl+C → 中断当前命令
                                              └─ 其他 → 追加到输入缓冲区
```

#### Shell 提示符

```
MiniOS [pid:1] /home $ _
```

格式: `MiniOS [pid:当前PID] 当前路径 $ `

---

## 6. 核心特色：Trace 追踪系统

### 6.1 Trace 引擎

#### 设计理念

借鉴 OpenTelemetry 的 Trace 模型，但针对内核场景做了深度定制：

- **零分配 Trace**: 所有 trace 数据存储在预分配的 Ring Buffer 中，不触发堆分配
- **纳秒级精度**: 使用 TSC (Time Stamp Counter) 作为时间源
- **跨模块关联**: 通过 TraceContext 自动传播 trace_id 和 parent_span_id
- **跨进程传播**: IPC 消息携带 TraceContext，实现跨进程链路追踪
- **最小性能影响**: 关键路径上的 trace 开销 < 100ns

#### 核心数据结构

```rust
/// 全局唯一的 Trace ID，标识一条完整调用链
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TraceId(pub u64);

/// Span ID，标识调用链中的一个操作
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanId(pub u64);

/// 一个 Trace Span — Trace 的基本单元
#[repr(C)]
pub struct Span {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub name: ArrayString<64>,        // 操作名称
    pub module: ArrayString<32>,       // 所属模块
    pub start_tsc: u64,                // 开始时间戳 (TSC)
    pub end_tsc: u64,                  // 结束时间戳 (TSC)
    pub status: SpanStatus,
    pub pid: Pid,                      // 产生此 span 的进程
    pub attributes: SpanAttributes,    // 键值对属性
}

pub enum SpanStatus {
    Ok,
    Error(ArrayString<128>),  // 错误信息
    InProgress,
}

/// Span 的附加属性（固定大小，零分配）
pub struct SpanAttributes {
    entries: [(ArrayString<32>, AttributeValue); 8], // 最多 8 个属性
    count: usize,
}

pub enum AttributeValue {
    U64(u64),
    I64(i64),
    Str(ArrayString<64>),
    Bool(bool),
}

/// Trace 上下文 — 随调用链传播
#[derive(Clone, Copy)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub current_span_id: SpanId,
    pub depth: u16,  // 嵌套深度（防止无限递归）
}
```

#### Ring Buffer 存储

```rust
pub struct TraceRingBuffer {
    buffer: &'static mut [Span],  // 固定大小数组
    capacity: usize,               // 最多存储的 span 数
    write_index: AtomicUsize,      // 写指针（原子操作，无需锁）
    total_written: AtomicU64,      // 总写入量（用于检测覆盖）
}
```

**配置**:

| 参数 | 值 |
|------|-----|
| Ring Buffer 大小 | 64K 个 Span |
| 单个 Span 大小 | ~512 字节 |
| 总内存消耗 | ~32 MiB |
| 写入方式 | 无锁原子写（覆盖最旧的 span） |
| 最大嵌套深度 | 64 层 |

#### Tracer Trait

```rust
pub trait Tracer: Send + Sync {
    /// 开始一个新的 span
    fn begin_span(&self, name: &str, module: &str) -> SpanGuard;

    /// 在当前 span 上添加事件
    fn add_event(&self, name: &str, attrs: &[(&str, AttributeValue)]);

    /// 获取当前的 trace 上下文
    fn current_context(&self) -> Option<TraceContext>;

    /// 设置当前的 trace 上下文（用于跨进程传播）
    fn set_context(&self, ctx: TraceContext);

    /// 清除当前 trace 上下文
    fn clear_context(&self);

    /// 读取 ring buffer 中的 span 数据
    fn read_spans(&self, filter: &SpanFilter, out: &mut [Span]) -> usize;

    /// 获取统计信息
    fn stats(&self) -> TraceStats;

    /// 清空 ring buffer
    fn clear(&self);

    /// 配置 trace 级别/过滤器
    fn configure(&self, config: TraceConfig);
}

/// RAII Guard — 在 drop 时自动结束 span
pub struct SpanGuard {
    span_id: SpanId,
    tracer: &'static dyn Tracer,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        self.tracer.end_span(self.span_id);
    }
}
```

#### Trace 宏

```rust
/// 创建一个 trace span，返回 RAII guard
#[macro_export]
macro_rules! trace_span {
    ($name:expr, module = $module:expr $(, $key:ident = $value:expr)*) => {{
        let guard = kernel().tracer.begin_span($name, $module);
        $(
            kernel().tracer.add_event(
                concat!(stringify!($key), "_set"),
                &[(stringify!($key), AttributeValue::from($value))]
            );
        )*
        guard
    }};
}

/// 记录一个瞬时事件
#[macro_export]
macro_rules! trace_event {
    ($name:expr $(, $key:ident = $value:expr)*) => {
        kernel().tracer.add_event($name, &[
            $( (stringify!($key), AttributeValue::from($value)), )*
        ]);
    };
}

/// 函数级 trace 注解（过程宏）
/// 自动在函数入口/出口创建 span
#[proc_macro_attribute]
pub fn traced(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 生成：
    // fn original_fn(args...) -> Ret {
    //     let _span = trace_span!("original_fn", module = "...");
    //     { 原始函数体 }
    // }
}
```

#### 使用示例

```rust
// 方式一：宏
fn handle_page_fault(addr: VirtAddr) {
    let _span = trace_span!("page_fault", module = "memory",
        address = addr.as_u64(),
        pid = current_pid()
    );

    let frame = kernel().memory.allocate_frame()
        .expect("out of memory");
    trace_event!("frame_allocated", frame = frame.number());

    kernel().memory.map_page(addr, frame, PageFlags::WRITABLE)
        .expect("map failed");
    trace_event!("page_mapped");
}

// 方式二：属性宏
#[traced(module = "scheduler")]
fn schedule_next() -> Option<Pid> {
    // 函数体自动被 trace span 包裹
    let next = self.ready_queue.pop_front();
    trace_event!("next_task", pid = next.unwrap_or(Pid(0)).0);
    next
}
```

### 6.2 Trace 日志格式

#### 二进制格式（Ring Buffer 内部存储）

```
┌─────────┬──────────┬───────────────┬──────────┬──────────┬────────────┬─────────────────┐
│TraceId  │ SpanId   │ ParentSpanId  │ StartTSC │ EndTSC   │ Module:Name│ Attributes      │
│ 8 bytes │ 8 bytes  │ 8+1 bytes     │ 8 bytes  │ 8 bytes  │ 32+64 bytes│ variable        │
└─────────┴──────────┴───────────────┴──────────┴──────────┴────────────┴─────────────────┘
```

#### JSON 导出格式（串口输出/文件导出）

```json
{
  "format": "minios-trace-v1",
  "exported_at": 1234567890,
  "tsc_frequency_hz": 2400000000,
  "spans": [
    {
      "trace_id": "0x00000000000ABC01",
      "span_id": "0x0000000000000001",
      "parent_span_id": null,
      "name": "kernel_boot",
      "module": "boot",
      "start_tsc": 1000000,
      "end_tsc": 5000000,
      "duration_ns": 1666,
      "status": "ok",
      "pid": 0,
      "attributes": {}
    },
    {
      "trace_id": "0x00000000000ABC01",
      "span_id": "0x0000000000000002",
      "parent_span_id": "0x0000000000000001",
      "name": "memory_init",
      "module": "memory",
      "start_tsc": 1100000,
      "end_tsc": 2500000,
      "duration_ns": 583,
      "status": "ok",
      "pid": 0,
      "attributes": {
        "total_frames": 262144,
        "free_frames": 260000
      }
    }
  ]
}
```

### 6.3 内核内 Trace 查看器

在 Shell 中通过 `trace` 命令系列访问。

#### `trace list` — 列表视图

```
MiniOS [pid:1] / $ trace list -n 5

 TRACE ID          SPAN ID   PARENT    MODULE      NAME               DURATION   STATUS
 ──────────────────────────────────────────────────────────────────────────────────────────
 0x00000ABC0001    0x0045    0x0044    memory      frame_alloc        120 ns     OK
 0x00000ABC0001    0x0044    0x0040    process     create_process     3.2 μs     OK
 0x00000DEF0001    0x0050    -         syscall     sys_write          850 ns     OK
 0x00000DEF0001    0x0051    0x0050    fs          vfs_write          600 ns     OK
 0x00000DEF0001    0x0052    0x0051    driver      serial_write       200 ns     OK

 Total spans in buffer: 1,234 / 65,536 (1.9%)
```

#### `trace tree` — 树形链路视图

```
MiniOS [pid:1] / $ trace tree 0x00000ABC0001

 Trace 0x00000ABC0001 — 12 spans, total: 15.3 μs
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 ├─ [boot] kernel_boot ·····················  15.3 μs  OK
 │  ├─ [boot] hal_init ·····················   1.2 μs  OK
 │  ├─ [boot] tracer_init ··················   0.5 μs  OK
 │  ├─ [memory] memory_init ················   3.8 μs  OK
 │  │  ├─ [memory] frame_allocator_init ····   2.1 μs  OK
 │  │  ├─ [memory] page_table_init ·········   1.2 μs  OK
 │  │  └─ [memory] heap_init ··············    0.5 μs  OK
 │  ├─ [interrupt] idt_init ················   0.8 μs  OK
 │  ├─ [process] process_mgr_init ··········   1.5 μs  OK
 │  ├─ [scheduler] scheduler_init ··········   0.3 μs  OK
 │  ├─ [fs] vfs_init ······················   2.2 μs  OK
 │  │  ├─ [fs] ramfs_create ···············   1.8 μs  OK
 │  │  └─ [fs] mount_root ·················   0.4 μs  OK
 │  └─ [process] create_init ··············   1.0 μs  OK
```

#### `trace live` — 实时流模式

```
MiniOS [pid:1] / $ trace live

 [LIVE TRACE] Press Ctrl+C to stop
 ──────────────────────────────────────────────────────
 12:00:01.001  ├─ [syscall] sys_read       pid=1  380 ns  OK
 12:00:01.002  ├─ [keyboard] key_event     pid=1  120 ns  OK
 12:00:01.010  ├─ [scheduler] tick         pid=0   80 ns  OK
 12:00:01.020  ├─ [scheduler] tick         pid=0   75 ns  OK
 12:00:01.025  ├─ [syscall] sys_write      pid=1  920 ns  OK
 12:00:01.025  │  └─ [fs] vfs_write                600 ns  OK
 12:00:01.025  │     └─ [driver] vga_write          200 ns  OK
 ^C
 Stopped. 7 spans captured.
```

### 6.4 宿主机可视化工具

#### 架构

```
┌──────────────────┐        串口输出         ┌─────────────────────┐
│    MiniOS 内核    │ ═══════════════════════►│  QEMU 串口 → 文件   │
│  Trace Engine    │   JSON trace data       │  (serial.log)       │
└──────────────────┘                         └──────────┬──────────┘
                                                        │
                                              ┌─────────▼──────────┐
                                              │   trace-viewer/    │
                                              │   index.html       │
                                              │                    │
                                              │  ┌──────────────┐  │
                                              │  │   瀑布图      │  │
                                              │  │  (Waterfall)  │  │
                                              │  └──────────────┘  │
                                              │  ┌──────────────┐  │
                                              │  │   火焰图      │  │
                                              │  │  (FlameGraph) │  │
                                              │  └──────────────┘  │
                                              │  ┌──────────────┐  │
                                              │  │   时间线      │  │
                                              │  │  (Timeline)   │  │
                                              │  └──────────────┘  │
                                              │  ┌──────────────┐  │
                                              │  │  模块依赖图   │  │
                                              │  │  (Dep Graph)  │  │
                                              │  └──────────────┘  │
                                              └────────────────────┘
```

#### 可视化视图

##### 视图 1: 瀑布图 (Waterfall)

最核心的视图，展示 span 的时间线和层级关系：

```
时间轴  0μs          5μs          10μs         15μs
        │            │            │            │
boot    ████████████████████████████████████████  kernel_boot (15.3μs)
          ██████  hal_init (1.2μs)
            ███  tracer_init (0.5μs)
              ████████████████  memory_init (3.8μs)
                ██████████  frame_alloc_init (2.1μs)
                      ██████  page_table_init (1.2μs)
                          ███  heap_init (0.5μs)
                            ██████  idt_init (0.8μs)
                                ████████  process_init (1.5μs)
```

**交互功能**:
- 鼠标悬停显示 span 详细信息（attributes）
- 点击 span 展开/折叠子 span
- 滚轮缩放时间轴
- 按模块着色
- 搜索/过滤 span

##### 视图 2: 火焰图 (Flame Graph)

```
┌──────────────────────────── kernel_boot ────────────────────────────┐
│ ┌─ hal_init ─┐┌tracer┐┌──── memory_init ────┐┌─ idt ─┐┌─ proc ──┐│
│ └────────────┘└──────┘│┌frame┐┌page ┐┌heap┐ │└───────┘└─────────┘│
│                       │└─────┘└─────┘└────┘  │                    │
│                       └──────────────────────┘                    │
└───────────────────────────────────────────────────────────────────┘
```

##### 视图 3: 时间线 (Timeline)

按进程分行展示活动：

```
        时间 →
PID 0  ████░░░░████░░░░████  (kernel idle + ticks)
PID 1  ░░░░████░░░░████░░░░  (shell 命令执行)
PID 2  ░░░░░░░░░░██░░░░░░░░  (后台任务)
       ──┬──┬──┬──┬──┬──┬──
         │  │  │  │  │  │
         调度切换点
```

##### 视图 4: 模块调用关系图

```
         syscall
        ╱       ╲
    process      fs
       │        ╱  ╲
    scheduler ramfs  vfs
       │
    memory
       │
      hal
```

边的粗细表示调用频率，颜色表示平均耗时。

#### HTML/JS 实现要点

```
trace-viewer/
├── index.html          # 主页面
├── css/
│   └── style.css       # 样式（暗色主题，适合技术人员）
├── js/
│   ├── main.js         # 入口，数据加载
│   ├── parser.js       # 解析 JSON trace 数据
│   ├── waterfall.js    # 瀑布图渲染 (Canvas)
│   ├── flamegraph.js   # 火焰图渲染 (Canvas)
│   ├── timeline.js     # 时间线渲染 (Canvas)
│   ├── depgraph.js     # 模块依赖图 (SVG)
│   └── utils.js        # 工具函数
└── sample/
    └── sample-trace.json  # 示例数据，方便开发调试
```

**数据加载方式**:
1. 文件拖拽：将 `serial.log` 拖拽到页面
2. 文件选择：通过文件选择器加载
3. URL 参数：`?file=path/to/trace.json`

---

## 7. 解耦设计详述

### 7.1 模块边界

每个模块是一个独立的 Rust crate，仅通过 trait 对外暴露接口：

| 模块 Crate | 暴露的 Trait | 依赖的 Trait |
|------------|-------------|-------------|
| `minios-hal` | `HalSerial`, `HalDisplay`, `HalInterruptController` | 无（最底层） |
| `minios-trace` | `Tracer` | `HalSerial`（输出通道） |
| `minios-memory` | `FrameAllocator`, `VirtualMemoryManager`, `HeapAllocator` | `Tracer` |
| `minios-interrupt` | `InterruptManager` | `HalInterruptController`, `Tracer` |
| `minios-process` | `ProcessManager` | `FrameAllocator`, `VirtualMemoryManager`, `Tracer` |
| `minios-scheduler` | `Scheduler` | `Tracer` |
| `minios-fs` | `FileSystem`, `FileSystemDriver` | `FrameAllocator`, `Tracer` |
| `minios-ipc` | `IpcManager` | `ProcessManager`, `Tracer` |
| `minios-syscall` | `SyscallDispatcher` | `ProcessManager`, `FileSystem`, `IpcManager`, `Tracer` |
| `minios-shell` | (无 trait, 顶层) | `SyscallDispatcher`, `Tracer` |
| `minios-kernel` | (集成 crate) | 所有上述 crate |

### 7.2 Workspace 组织

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/hal",
    "crates/trace",
    "crates/trace-macros",
    "crates/memory",
    "crates/interrupt",
    "crates/process",
    "crates/scheduler",
    "crates/fs",
    "crates/ipc",
    "crates/syscall",
    "crates/shell",
    "crates/kernel",
]
```

### 7.3 接口替换示例

通过 feature flag 切换实现：

```toml
# crates/memory/Cargo.toml
[features]
default = ["bitmap-allocator"]
bitmap-allocator = []     # 默认: Bitmap 帧分配器
buddy-allocator = []      # 可选: Buddy System 帧分配器
```

```rust
// crates/memory/src/lib.rs
#[cfg(feature = "bitmap-allocator")]
mod bitmap;
#[cfg(feature = "bitmap-allocator")]
pub use bitmap::BitmapFrameAllocator as DefaultFrameAllocator;

#[cfg(feature = "buddy-allocator")]
mod buddy;
#[cfg(feature = "buddy-allocator")]
pub use buddy::BuddyFrameAllocator as DefaultFrameAllocator;
```

### 7.4 依赖注入模式

```rust
// 初始化时注入依赖
pub fn init_process_manager(
    memory: &'static dyn FrameAllocator,
    vmm: &'static dyn VirtualMemoryManager,
    tracer: &'static dyn Tracer,
) -> impl ProcessManager {
    ProcessManagerImpl::new(memory, vmm, tracer)
}

// ProcessManagerImpl 内部只持有 trait 引用
struct ProcessManagerImpl {
    memory: &'static dyn FrameAllocator,
    vmm: &'static dyn VirtualMemoryManager,
    tracer: &'static dyn Tracer,
    processes: SpinLock<BTreeMap<Pid, Process>>,
}
```

### 7.5 测试中的 Mock

```rust
// 任何模块都可以使用 mock 实现进行测试
#[cfg(test)]
mod tests {
    struct MockFrameAllocator {
        next_frame: AtomicU64,
    }

    impl FrameAllocator for MockFrameAllocator {
        fn allocate_frame(&self) -> Option<PhysFrame> {
            let n = self.next_frame.fetch_add(1, Ordering::SeqCst);
            Some(PhysFrame::from_number(n))
        }
        fn deallocate_frame(&self, _frame: PhysFrame) {}
        fn free_frames(&self) -> usize { usize::MAX }
        fn total_frames(&self) -> usize { usize::MAX }
    }

    #[test]
    fn test_create_process() {
        let mock_alloc = MockFrameAllocator::new();
        let mock_tracer = NullTracer; // 不记录 trace 的空实现
        let pm = ProcessManagerImpl::new(&mock_alloc, &mock_tracer);
        let pid = pm.create_process("test", test_fn).unwrap();
        assert_eq!(pid, Pid(1));
    }
}
```

---

## 8. 目录结构

```
MiniOperationSystem/
├── Cargo.toml                    # Workspace 定义
├── Makefile.toml                 # cargo-make 任务定义
├── rust-toolchain.toml           # Rust nightly 工具链锁定
├── spec.md                       # 本规格文档
├── README.md                     # 项目简介
├── AGENTS.md                     # AI Agent 开发指引
│
├── crates/                       # 内核模块 (每个模块一个 crate)
│   ├── hal/                      # 硬件抽象层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # HAL 公共接口
│   │       ├── port.rs           # 端口 I/O
│   │       ├── cpu.rs            # CPU 操作
│   │       ├── serial.rs         # 串口驱动
│   │       ├── vga.rs            # VGA 文本模式
│   │       ├── pic.rs            # PIC 中断控制器
│   │       └── gdt.rs            # GDT/TSS
│   │
│   ├── trace/                    # Trace 引擎
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Tracer trait + 公共接口
│   │       ├── engine.rs         # 核心引擎实现
│   │       ├── ringbuffer.rs     # Ring Buffer 存储
│   │       ├── context.rs        # TraceContext 管理
│   │       ├── span.rs           # Span 数据结构
│   │       ├── export.rs         # JSON 导出
│   │       └── filter.rs         # 过滤器
│   │
│   ├── trace-macros/             # Trace 过程宏
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # #[traced] 属性宏
│   │
│   ├── memory/                   # 内存管理
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 公共接口 (FrameAllocator, VMM)
│   │       ├── frame/
│   │       │   ├── mod.rs
│   │       │   ├── bitmap.rs     # Bitmap 帧分配器
│   │       │   └── buddy.rs      # Buddy 帧分配器（可选）
│   │       ├── paging/
│   │       │   ├── mod.rs
│   │       │   ├── table.rs      # 页表操作
│   │       │   └── mapper.rs     # 页面映射
│   │       └── heap/
│   │           ├── mod.rs
│   │           └── linked_list.rs # 链表堆分配器
│   │
│   ├── interrupt/                # 中断管理
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── idt.rs            # IDT 设置
│   │       ├── handlers.rs       # 异常处理函数
│   │       └── irq.rs            # 硬件中断处理
│   │
│   ├── process/                  # 进程管理
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pcb.rs            # 进程控制块
│   │       ├── manager.rs        # 进程管理器
│   │       ├── context.rs        # CPU 上下文 + 切换
│   │       └── pid.rs            # PID 分配器
│   │
│   ├── scheduler/                # 调度器
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── mlfq.rs           # MLFQ 调度器实现
│   │
│   ├── fs/                       # 文件系统
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # VFS trait
│   │       ├── vfs.rs            # VFS 实现
│   │       ├── ramfs.rs          # RamFS 实现
│   │       ├── procfs.rs         # ProcFS 实现
│   │       ├── tracefs.rs        # TraceFS 实现
│   │       ├── devfs.rs          # DevFS 实现
│   │       └── fd.rs             # 文件描述符表
│   │
│   ├── ipc/                      # 进程间通信
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── message.rs        # 消息队列
│   │       └── shared_mem.rs     # 共享内存
│   │
│   ├── syscall/                  # 系统调用
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dispatcher.rs     # 分发器
│   │       ├── io.rs             # I/O 相关 syscall
│   │       ├── process.rs        # 进程相关 syscall
│   │       ├── memory.rs         # 内存相关 syscall
│   │       └── trace.rs          # Trace 相关 syscall
│   │
│   ├── shell/                    # Shell
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # 命令解析
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── fs_cmds.rs    # ls, cd, cat, mkdir...
│   │       │   ├── proc_cmds.rs  # ps, kill
│   │       │   ├── sys_cmds.rs   # meminfo, uptime, help
│   │       │   └── trace_cmds.rs # trace list/tree/live/export
│   │       └── input.rs          # 输入处理
│   │
│   └── kernel/                   # 内核集成
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # 入口点 (_start / kernel_main)
│           ├── services.rs       # KernelServices 注册表
│           └── panic.rs          # Panic handler
│
├── trace-viewer/                 # 宿主机 Web 可视化工具
│   ├── index.html
│   ├── css/
│   │   └── style.css
│   ├── js/
│   │   ├── main.js
│   │   ├── parser.js
│   │   ├── waterfall.js
│   │   ├── flamegraph.js
│   │   ├── timeline.js
│   │   ├── depgraph.js
│   │   └── utils.js
│   └── sample/
│       └── sample-trace.json
│
├── tests/                        # 集成测试
│   ├── boot_test.rs              # 引导测试
│   ├── memory_test.rs            # 内存测试
│   ├── process_test.rs           # 进程测试
│   └── trace_test.rs             # Trace 测试
│
└── scripts/
    ├── run.sh                    # 快速启动脚本
    ├── debug.sh                  # GDB 调试脚本
    └── capture-trace.sh          # 从串口捕获 trace 数据
```

---

## 9. 构建系统

### 9.1 rust-toolchain.toml

```toml
[toolchain]
channel = "nightly"
components = ["rust-src", "llvm-tools-preview", "rustfmt", "clippy"]
targets = ["x86_64-unknown-none"]
```

### 9.2 Makefile.toml (cargo-make)

```toml
[tasks.build]
description = "编译内核"
command = "cargo"
args = ["build", "--target", "x86_64-unknown-none", "--release"]

[tasks.run]
description = "编译并在 QEMU 中运行"
dependencies = ["build"]
script = '''
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/boot-bios-minios.img \
    -serial stdio \
    -display gtk \
    -m 128M \
    -no-reboot \
    -no-shutdown
'''

[tasks.run-headless]
description = "无界面运行（仅串口输出）"
dependencies = ["build"]
script = '''
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/boot-bios-minios.img \
    -serial stdio \
    -display none \
    -m 128M \
    -no-reboot
'''

[tasks.run-trace]
description = "运行并捕获 trace 到文件"
dependencies = ["build"]
script = '''
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/boot-bios-minios.img \
    -serial file:trace-output.log \
    -display gtk \
    -m 128M \
    -no-reboot
'''

[tasks.test]
description = "运行所有测试"
command = "cargo"
args = ["test"]

[tasks.test-integration]
description = "运行集成测试（需要 QEMU）"
command = "cargo"
args = ["test", "--test", "*", "--target", "x86_64-unknown-none"]

[tasks.clippy]
description = "代码检查"
command = "cargo"
args = ["clippy", "--all-targets", "--", "-D", "warnings"]

[tasks.fmt]
description = "格式化代码"
command = "cargo"
args = ["fmt", "--all"]

[tasks.fmt-check]
description = "检查代码格式"
command = "cargo"
args = ["fmt", "--all", "--check"]

[tasks.doc]
description = "生成文档"
command = "cargo"
args = ["doc", "--no-deps", "--open"]

[tasks.clean]
description = "清理构建产物"
command = "cargo"
args = ["clean"]

[tasks.debug]
description = "启动 QEMU 并等待 GDB 连接"
dependencies = ["build"]
script = '''
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/boot-bios-minios.img \
    -serial stdio \
    -m 128M \
    -s -S \
    -no-reboot
'''
```

### 9.3 常用命令汇总

```bash
# 编译
cargo make build

# 编译并运行
cargo make run

# 运行测试
cargo make test

# 代码检查
cargo make clippy

# 格式化
cargo make fmt

# 运行并捕获 trace
cargo make run-trace

# 调试
cargo make debug  # 终端1: 启动 QEMU
gdb target/...    # 终端2: 连接 GDB
```

---

## 10. 测试策略

### 10.1 测试层级

| 层级 | 范围 | 方法 | 工具 |
|------|------|------|------|
| 单元测试 | 每个模块的内部逻辑 | `#[test]` + Mock trait 实现 | `cargo test` |
| 集成测试 | 模块间交互 | `#[test_case]` 在 QEMU 中运行 | `cargo test --test` |
| 引导测试 | 内核能否正常引导 | QEMU + 串口输出检查 | 自定义测试框架 |
| Trace 测试 | Trace 链路完整性 | 导出 trace → 验证 span 关系 | JSON 校验脚本 |

### 10.2 自定义测试框架

由于 `#![no_std]` 环境无法使用标准测试框架，需要自定义：

```rust
// 在 crates/kernel/src/main.rs 中
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reboot_on_panic]

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
```

### 10.3 关键测试用例

#### 内存测试

```rust
#[test_case]
fn test_frame_alloc_dealloc() {
    let frame = kernel().memory.allocate_frame().unwrap();
    kernel().memory.deallocate_frame(frame);
}

#[test_case]
fn test_page_mapping() {
    let page = Page::containing_address(VirtAddr::new(0xDEAD_BEEF_0000));
    let frame = kernel().memory.allocate_frame().unwrap();
    kernel().memory.map_page(page, frame, PageFlags::WRITABLE).unwrap();
    let translated = kernel().memory.translate_addr(page.start_address());
    assert_eq!(translated, Some(frame.start_address()));
}

#[test_case]
fn test_heap_allocation() {
    let v = alloc::vec![1, 2, 3, 4, 5];
    assert_eq!(v.len(), 5);
    assert_eq!(v[2], 3);
}
```

#### 进程测试

```rust
#[test_case]
fn test_create_process() {
    let pid = kernel().process.create_process("test", || {
        // 简单的测试任务
    }).unwrap();
    assert!(pid.0 > 0);
}

#[test_case]
fn test_process_list() {
    let procs = kernel().process.list_processes();
    assert!(procs.len() >= 1); // 至少有 init 进程
}
```

#### Trace 测试

```rust
#[test_case]
fn test_trace_span_creation() {
    let _span = trace_span!("test_span", module = "test");
    // span 应该在 ring buffer 中
    let mut buf = [Span::default(); 1];
    let count = kernel().tracer.read_spans(
        &SpanFilter::by_name("test_span"),
        &mut buf,
    );
    assert!(count >= 1);
}

#[test_case]
fn test_trace_parent_child() {
    let _parent = trace_span!("parent", module = "test");
    let _child = trace_span!("child", module = "test");
    // child 的 parent_span_id 应该等于 parent 的 span_id
}

#[test_case]
fn test_trace_cross_process() {
    // 创建两个进程，通过 IPC 通信
    // 验证 trace_id 在消息传递后保持一致
}
```

### 10.4 CI 流程

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rustfmt, clippy, rust-src, llvm-tools-preview
          targets: x86_64-unknown-none
      - name: Format check
        run: cargo fmt --all --check
      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: Unit tests
        run: cargo test --workspace
      - name: Install QEMU
        run: sudo apt-get install -y qemu-system-x86
      - name: Integration tests
        run: cargo test --test '*' --target x86_64-unknown-none
```

---

## 11. 开发阶段规划

### Phase 0: 项目脚手架 (预计: 1 session)

- [x] 创建 README.md
- [ ] 初始化 Cargo workspace
- [ ] 配置 rust-toolchain.toml
- [ ] 配置 Makefile.toml
- [ ] 创建所有 crate 骨架（空 lib.rs + Cargo.toml）
- [ ] 验证 `cargo build` 通过

### Phase 1: 引导 + HAL (预计: 1-2 sessions)

- [ ] 实现 `crates/hal`: 串口、VGA、端口 I/O、GDT
- [ ] 实现 `crates/kernel/src/main.rs`: 入口点
- [ ] 引导到 VGA 输出 "Hello, MiniOS!"
- [ ] 串口输出工作
- [ ] `cargo make run` 可在 QEMU 中启动
- **验收**: QEMU 窗口显示 "Hello, MiniOS!"，串口输出引导日志

### Phase 2: Trace 引擎 (预计: 1-2 sessions)

- [ ] 实现 `crates/trace`: Ring Buffer、Span、TraceContext
- [ ] 实现 `crates/trace-macros`: `#[traced]` 属性宏
- [ ] 实现 `trace_span!` 和 `trace_event!` 宏
- [ ] 集成到引导流程，每步初始化生成 trace
- [ ] 串口输出 trace 数据
- **验收**: 引导过程中串口输出结构化 trace 数据

### Phase 3: 内存管理 (预计: 2 sessions)

- [ ] 实现物理帧分配器（Bitmap）
- [ ] 实现页表管理（映射/取消映射/地址翻译）
- [ ] 实现内核堆分配器（Linked List）
- [ ] 所有操作都有 trace
- [ ] 单元测试通过
- **验收**: 可以使用 `alloc::vec!`，内存操作有完整 trace 链

### Phase 4: 中断处理 (预计: 1 session)

- [ ] 实现 IDT 设置
- [ ] 实现异常处理（Page Fault, Double Fault 等）
- [ ] 实现 PIC 初始化
- [ ] 实现时钟中断处理
- [ ] 实现键盘中断处理
- [ ] 所有中断处理有 trace
- **验收**: 键盘输入在 VGA 上回显，时钟中断正常 tick

### Phase 5: 进程管理 + 调度器 (预计: 2-3 sessions)

- [ ] 实现 PCB 和进程管理器
- [ ] 实现上下文切换（汇编）
- [ ] 实现 MLFQ 调度器
- [ ] 创建 idle 进程和 init 进程
- [ ] 进程切换有完整 trace
- **验收**: 多个内核任务并发执行，trace 显示调度决策和切换

### Phase 6: 文件系统 (预计: 2 sessions)

- [ ] 实现 VFS 层
- [ ] 实现 RamFS
- [ ] 实现 ProcFS
- [ ] 实现 TraceFS
- [ ] 实现 DevFS
- [ ] 文件操作有 trace
- **验收**: 可以 create/read/write/delete 文件，`/proc/meminfo` 返回正确数据

### Phase 7: 系统调用 (预计: 1-2 sessions)

- [ ] 实现系统调用分发器
- [ ] 实现所有基础系统调用
- [ ] 系统调用有 trace
- **验收**: 进程可通过 syscall 进行 I/O 和进程管理

### Phase 8: IPC (预计: 1 session)

- [ ] 实现消息队列
- [ ] 实现 trace 上下文跨进程传播
- **验收**: 两个进程通过消息队列通信，trace 链路跨进程连贯

### Phase 9: Shell (预计: 2 sessions)

- [ ] 实现命令解析器
- [ ] 实现所有基础命令 (ls, cd, cat, ps, etc.)
- [ ] 实现 trace 命令系列 (trace list/tree/live/export)
- **验收**: Shell 交互流畅，trace 命令显示正确的追踪链路

### Phase 10: 宿主机可视化工具 (预计: 2 sessions)

- [ ] 实现 JSON trace 解析器
- [ ] 实现瀑布图
- [ ] 实现火焰图
- [ ] 实现时间线视图
- [ ] 实现模块依赖图
- [ ] 交互功能（缩放、搜索、过滤）
- **验收**: 导出 trace → 在浏览器中看到完整的可视化链路

### Phase 11: 集成测试 + 打磨 (预计: 1-2 sessions)

- [ ] 全链路测试：引导 → Shell → 执行命令 → trace 导出 → 可视化
- [ ] 性能优化（trace 开销 < 100ns）
- [ ] 错误处理完善
- [ ] 文档补全
- **验收**: 完整的端到端演示视频

---

## 12. 关键接口定义

### 12.1 完整 Trait 一览

以下是所有模块的核心 trait 汇总，这些 trait 构成了整个系统的解耦边界：

```rust
// ═══════════════════════════════════════════════════════
// HAL 层
// ═══════════════════════════════════════════════════════

pub trait HalSerial: Send + Sync {
    fn write_byte(&self, byte: u8);
    fn write_bytes(&self, bytes: &[u8]);
    fn read_byte(&self) -> Option<u8>;
}

pub trait HalDisplay: Send + Sync {
    fn write_char(&self, row: usize, col: usize, ch: u8, color: ColorCode);
    fn scroll_up(&self);
    fn clear(&self);
    fn dimensions(&self) -> (usize, usize);
}

pub trait HalInterruptController: Send + Sync {
    fn init(&self);
    fn enable_irq(&self, irq: u8);
    fn disable_irq(&self, irq: u8);
    fn end_of_interrupt(&self, irq: u8);
}

// ═══════════════════════════════════════════════════════
// Trace 层
// ═══════════════════════════════════════════════════════

pub trait Tracer: Send + Sync {
    fn begin_span(&self, name: &str, module: &str) -> SpanGuard;
    fn end_span(&self, span_id: SpanId);
    fn add_event(&self, name: &str, attrs: &[(&str, AttributeValue)]);
    fn current_context(&self) -> Option<TraceContext>;
    fn set_context(&self, ctx: TraceContext);
    fn clear_context(&self);
    fn read_spans(&self, filter: &SpanFilter, out: &mut [Span]) -> usize;
    fn stats(&self) -> TraceStats;
    fn clear(&self);
    fn configure(&self, config: TraceConfig);
}

// ═══════════════════════════════════════════════════════
// 内存层
// ═══════════════════════════════════════════════════════

pub trait FrameAllocator: Send + Sync {
    fn allocate_frame(&self) -> Option<PhysFrame>;
    fn deallocate_frame(&self, frame: PhysFrame);
    fn free_frames(&self) -> usize;
    fn total_frames(&self) -> usize;
}

pub trait VirtualMemoryManager: Send + Sync {
    fn map_page(&self, page: Page, frame: PhysFrame, flags: PageFlags,
                alloc: &dyn FrameAllocator) -> Result<(), MapError>;
    fn unmap_page(&self, page: Page) -> Result<PhysFrame, UnmapError>;
    fn translate_addr(&self, virt: VirtAddr) -> Option<PhysAddr>;
    fn create_address_space(&self) -> Result<AddressSpace, MemoryError>;
    fn switch_address_space(&self, space: &AddressSpace);
}

pub trait HeapAllocator: Send + Sync {
    fn allocate(&self, layout: Layout) -> *mut u8;
    fn deallocate(&self, ptr: *mut u8, layout: Layout);
    fn used_bytes(&self) -> usize;
    fn free_bytes(&self) -> usize;
}

// ═══════════════════════════════════════════════════════
// 进程层
// ═══════════════════════════════════════════════════════

pub trait ProcessManager: Send + Sync {
    fn create_process(&self, name: &str, entry: fn()) -> Result<Pid, ProcessError>;
    fn exit_process(&self, pid: Pid, code: i32) -> Result<(), ProcessError>;
    fn kill_process(&self, pid: Pid) -> Result<(), ProcessError>;
    fn get_process(&self, pid: Pid) -> Option<&Process>;
    fn current_pid(&self) -> Pid;
    fn list_processes(&self) -> Vec<ProcessInfo>;
    fn fork(&self) -> Result<Pid, ProcessError>;
    fn waitpid(&self, pid: Pid) -> Result<i32, ProcessError>;
}

// ═══════════════════════════════════════════════════════
// 调度器
// ═══════════════════════════════════════════════════════

pub trait Scheduler: Send + Sync {
    fn add_task(&self, pid: Pid, priority: Priority);
    fn remove_task(&self, pid: Pid);
    fn tick(&self) -> ScheduleDecision;
    fn next_task(&self) -> Option<Pid>;
    fn yield_current(&self);
    fn block_current(&self, reason: BlockReason);
    fn unblock(&self, pid: Pid);
    fn set_priority(&self, pid: Pid, priority: Priority);
    fn stats(&self) -> SchedulerStats;
}

// ═══════════════════════════════════════════════════════
// 文件系统层
// ═══════════════════════════════════════════════════════

pub trait FileSystem: Send + Sync {
    fn open(&self, path: &str, flags: OpenFlags) -> Result<FileDescriptor, FsError>;
    fn close(&self, fd: FileDescriptor) -> Result<(), FsError>;
    fn read(&self, fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&self, fd: FileDescriptor, buf: &[u8]) -> Result<usize, FsError>;
    fn seek(&self, fd: FileDescriptor, offset: i64, whence: SeekWhence) -> Result<u64, FsError>;
    fn mkdir(&self, path: &str) -> Result<(), FsError>;
    fn rmdir(&self, path: &str) -> Result<(), FsError>;
    fn unlink(&self, path: &str) -> Result<(), FsError>;
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, FsError>;
    fn stat(&self, path: &str) -> Result<FileStat, FsError>;
    fn mount(&self, path: &str, fs: Box<dyn FileSystemDriver>) -> Result<(), FsError>;
}

pub trait FileSystemDriver: Send + Sync {
    fn name(&self) -> &str;
    fn create_file(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn create_dir(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn read_data(&self, inode: InodeId, off: usize, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write_data(&self, inode: InodeId, off: usize, buf: &[u8]) -> Result<usize, FsError>;
    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn remove(&self, parent: InodeId, name: &str) -> Result<(), FsError>;
    fn list(&self, inode: InodeId) -> Result<Vec<DirEntry>, FsError>;
    fn stat(&self, inode: InodeId) -> Result<FileStat, FsError>;
}

// ═══════════════════════════════════════════════════════
// IPC 层
// ═══════════════════════════════════════════════════════

pub trait IpcManager: Send + Sync {
    fn create_queue(&self, name: &str, cap: usize) -> Result<QueueId, IpcError>;
    fn send(&self, queue: QueueId, msg: &Message) -> Result<(), IpcError>;
    fn receive(&self, queue: QueueId, timeout: Option<u64>) -> Result<Message, IpcError>;
    fn destroy_queue(&self, queue: QueueId) -> Result<(), IpcError>;
    fn create_shm(&self, name: &str, size: usize) -> Result<ShmId, IpcError>;
    fn attach_shm(&self, shm: ShmId, pid: Pid) -> Result<VirtAddr, IpcError>;
    fn detach_shm(&self, shm: ShmId, pid: Pid) -> Result<(), IpcError>;
}

// ═══════════════════════════════════════════════════════
// 设备驱动
// ═══════════════════════════════════════════════════════

pub trait DeviceDriver: Send + Sync {
    fn name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
    fn init(&self) -> Result<(), DriverError>;
    fn read(&self, buf: &mut [u8]) -> Result<usize, DriverError>;
    fn write(&self, buf: &[u8]) -> Result<usize, DriverError>;
    fn ioctl(&self, cmd: u32, arg: usize) -> Result<usize, DriverError>;
}
```

### 12.2 错误类型

```rust
#[derive(Debug)]
pub enum KernelError {
    Memory(MemoryError),
    Process(ProcessError),
    FileSystem(FsError),
    Ipc(IpcError),
    Driver(DriverError),
    Trace(TraceError),
}

#[derive(Debug)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress,
    AlreadyMapped,
    NotMapped,
    AlignmentError,
}

#[derive(Debug)]
pub enum ProcessError {
    MaxProcessesReached,
    InvalidPid,
    ProcessNotFound,
    InvalidState,
    StackAllocationFailed,
}

#[derive(Debug)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    PermissionDenied,
    NoSpace,
    InvalidPath,
    TooManyOpenFiles,
    InvalidDescriptor,
}

#[derive(Debug)]
pub enum IpcError {
    QueueFull,
    QueueEmpty,
    QueueNotFound,
    Timeout,
    InvalidMessage,
}
```

---

## 13. 错误处理策略

### 13.1 分层错误处理

| 层级 | 错误处理方式 | 说明 |
|------|-------------|------|
| HAL | `panic!` | 硬件错误不可恢复 |
| Memory | `Result<T, MemoryError>` | 内存不足等可恢复 |
| Process | `Result<T, ProcessError>` | 进程操作可能失败 |
| FS | `Result<T, FsError>` | 文件操作可能失败 |
| Syscall | `i64` 返回值 | 负值表示错误码 |
| Shell | 打印错误消息 | 用户可见的错误提示 |

### 13.2 Panic 处理

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 1. 禁用中断
    hal::cpu::disable_interrupts();

    // 2. 输出 panic 信息到 VGA（红色背景）
    vga::set_color(Color::White, Color::Red);
    vga::print("!!! KERNEL PANIC !!!\n");
    vga::print(&format!("{}\n", info));

    // 3. 输出到串口（便于自动化测试捕获）
    serial::write("KERNEL PANIC: ");
    serial::write(&format!("{}\n", info));

    // 4. 导出当前 trace 上下文（帮助调试）
    if let Some(ctx) = tracer::current_context() {
        serial::write(&format!("Trace context: trace_id={:?} span_id={:?}\n",
            ctx.trace_id, ctx.current_span_id));
    }

    // 5. Halt
    loop { hal::cpu::hlt(); }
}
```

### 13.3 错误 Trace

每个错误都会生成一个带 `Error` 状态的 span：

```rust
fn sys_open(path: &str, flags: OpenFlags) -> Result<FileDescriptor, FsError> {
    let _span = trace_span!("sys_open", module = "syscall", path = path);

    match kernel().fs.open(path, flags) {
        Ok(fd) => {
            trace_event!("opened", fd = fd.0);
            Ok(fd)
        }
        Err(e) => {
            trace_event!("error", error = format!("{:?}", e));
            Err(e)
        }
    }
}
```

---

## 14. 性能约束与目标

### 14.1 Trace 开销

| 操作 | 目标 | 方法 |
|------|------|------|
| 创建 Span | < 100 ns | 预分配 Ring Buffer，无堆分配 |
| 添加 Event | < 50 ns | 固定大小属性数组 |
| 结束 Span | < 50 ns | 原子写入 Ring Buffer |
| 读取 Span | < 1 μs / span | 直接内存访问 |
| JSON 导出 | < 10 ms / 1000 spans | 流式序列化到串口 |

### 14.2 系统性能

| 指标 | 目标 |
|------|------|
| 引导到 Shell | < 500 ms (QEMU) |
| 上下文切换 | < 5 μs |
| 系统调用开销 | < 1 μs (不含实际操作) |
| 页面映射 | < 2 μs |
| 帧分配 | < 500 ns |
| Shell 命令响应 | < 10 ms |

### 14.3 内存使用

| 组件 | 内存预算 |
|------|---------|
| 内核代码 + 数据 | < 2 MiB |
| 内核堆 | 1-16 MiB（动态扩展） |
| Trace Ring Buffer | 32 MiB |
| 每进程内核栈 | 64 KiB |
| 每进程用户栈 | 1 MiB |
| 页表 | 按需分配 |
| **总计（最小）** | **~40 MiB** |
| **QEMU 建议内存** | **128 MiB** |

---

## 附录 A: 约定和编码规范

### A.1 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| Crate 名 | `minios-xxx` | `minios-memory` |
| Trait | PascalCase | `FrameAllocator` |
| Struct | PascalCase | `BitmapFrameAllocator` |
| 函数/方法 | snake_case | `allocate_frame()` |
| 常量 | SCREAMING_SNAKE | `MAX_PROCESSES` |
| Trace Span 名 | snake_case | `"frame_alloc"` |
| Trace Module 名 | 模块名 | `"memory"`, `"process"` |

### A.2 注释规范

- 所有公开 trait 和方法必须有 `///` rustdoc 注释
- 不安全代码（`unsafe`）必须有 `// SAFETY:` 注释解释安全性保证
- 复杂算法必须有行内注释解释逻辑

### A.3 Git 约定

```
feat(module): 简短描述
fix(module): 修复描述
refactor(module): 重构描述
test(module): 测试描述
docs: 文档描述

例:
feat(memory): implement bitmap frame allocator
feat(trace): add ring buffer storage
fix(scheduler): prevent priority inversion starvation
test(process): add context switch integration test
```

---

## 附录 B: 参考资料

| 资料 | 链接 | 用途 |
|------|------|------|
| Writing an OS in Rust | https://os.phil-opp.com/ | 基础参考 |
| OSDev Wiki | https://wiki.osdev.org/ | 硬件细节参考 |
| Intel SDM | https://www.intel.com/sdm | x86_64 指令集手册 |
| Rust `x86_64` crate | https://docs.rs/x86_64 | 页表、中断等 API |
| Rust `bootloader` crate | https://docs.rs/bootloader | 引导器 API |
| OpenTelemetry Spec | https://opentelemetry.io/docs/specs/ | Trace 模型参考 |

---

## 附录 C: Vibe Coding 指引

本规格设计为可完全通过 AI 辅助编码（Vibe Coding）实现。以下是给 AI Agent 的开发指引：

### C.1 开发顺序

**严格按照 Phase 0 → Phase 11 的顺序开发**。每个 Phase 都有明确的验收标准。

### C.2 每个 Phase 的工作流

1. 阅读该 Phase 的所有相关 Spec 章节
2. 创建所需的文件结构
3. 实现 Trait 定义（接口先行）
4. 实现具体逻辑
5. 为每个函数添加 trace instrumentation
6. 编写单元测试
7. `cargo build` + `cargo test` 通过
8. 在 QEMU 中验证
9. 提交代码

### C.3 决策规则

遇到规格中未明确的细节时：

1. 优先选择最简单的实现
2. 优先选择可测试的实现
3. 优先选择有 trace 覆盖的实现
4. 不确定时，在代码中添加 `// TODO: [decision needed]` 注释

### C.4 调试技巧

- 使用串口输出（`serial_println!`）进行基本调试
- 使用 Trace 系统查看调用链
- 使用 QEMU `-d` 选项查看 CPU 日志
- 使用 GDB 远程调试（`cargo make debug`）

---

*文档版本: 1.0*
*最后更新: 2026-02-25*
*状态: 初始规格，待开发*
