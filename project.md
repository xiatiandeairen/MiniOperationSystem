# MiniOS Project

## 项目定位
操作系统学习项目 + Trace 特性试点。帮助开发者理解 OS 运行机制，降低噪音信息理解成本。

## 项目状态
✅ **v0.1.0 已发布** — 可引导、可交互、可观测
🔜 v0.2.0 规划中 — "Teach by Running" (调度可操控 + Trace 教学化)

## 架构概览
13-crate Rust workspace, x86_64 bare-metal, bootloader_api v0.11

## 子系统状态

| 子系统 | 状态 | 核心能力 |
|--------|------|---------|
| HAL | ✅ | 串口 UART, 帧缓冲文本控制台(彩色), GDT/TSS, IDT, PIC, PS/2键盘 |
| Trace | ✅ | 4096-slot Ring Buffer, SpanGuard RAII, JSON export, 引导流程集成 |
| Memory | ✅ | Bitmap帧分配器(256MB), 4级页表, 1MiB链表堆 |
| Process | ✅ | PCB, 上下文切换汇编, PID分配器 |
| Scheduler | ✅ | MLFQ 4级队列, O(n) boost, 时间片管理 |
| FileSystem | ✅ | VFS + RamFS(inode-based) + ProcFS(实时数据) |
| IPC | ✅ | 消息队列 + TraceContext跨进程传播 |
| Syscall | ✅ | 7个syscall (read/write/getpid/yield/exit/uptime/meminfo) |
| Shell | ✅ | 15命令, 彩色提示符, trace tree/export, ls带文件大小 |
| Trace Viewer | ✅ | 瀑布图 Web 工具, 23-span 样例数据 |

## 质量指标
- 代码: 66 源文件, ~6400 行 Rust
- 测试: 59 单元测试 (common 28 + trace 18 + scheduler 8 + ipc 5)
- Lint: cargo clippy 零警告
- 格式: cargo fmt 通过

## 版本历史
- [v0.1.0](docs/releases/v0.1.0.md) — 首个可交互版本 "Hello, MiniOS!"
- [v0.2.0 Roadmap](docs/releases/v0.2.0-roadmap.md) — "Teach by Running"

## 文档索引
- [README.md](README.md) — 快速开始
- [spec.md](spec.md) — 技术规格 (2500行)
- [plan.md](plan.md) — 开发计划 (500任务, 10检查点)
- [claude.md](claude.md) — AI 自驱动开发规则
- [CHANGELOG.md](CHANGELOG.md) — 变更日志
- [docs/releases/](docs/releases/) — 版本发布文档
- [docs/tasks/](docs/tasks/) — 迭代计划和结果
- [docs/adr/](docs/adr/) — 架构决策记录
