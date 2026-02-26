# MiniOS Project

## 项目定位
操作系统学习项目 + Trace 特性试点。帮助开发者理解 OS 运行机制，降低噪音信息理解成本。

## 项目状态
- ✅ **v0.1.0** "Hello, MiniOS!" — 首个可交互版本
- ✅ **v0.2.0** "Teach by Running" — 8个教学命令
- ✅ **v1.0-rc** "Production Polish" — 全功能 shell，含 memmap/pstree/safety/report/man/snapshot/version

## 架构概览
13-crate Rust workspace, x86_64 bare-metal, bootloader_api v0.11

## 子系统状态

| 子系统 | 状态 | 核心能力 |
|--------|------|---------|
| HAL | ✅ | 串口, 帧缓冲控制台(彩色), GDT/TSS, IDT, PIC, 键盘 |
| Trace | ✅ | Ring Buffer, SpanGuard, JSON export, syscall+VFS 集成 |
| Memory | ✅ | Bitmap帧分配器, 4级页表, 1MiB堆 |
| Process | ✅ | PCB, 上下文切换汇编, PID分配器 |
| Scheduler | ✅ | MLFQ 4级, O(n) boost |
| FileSystem | ✅ | VFS + RamFS + ProcFS(实时数据) |
| IPC | ✅ | 消息队列 + TraceContext传播 |
| Syscall | ✅ | 7 calls (函数调用式) |
| Shell | ✅ | 23命令 (含 spawn/kill/sched/trace follow/pagetable/frames) |
| Trace Viewer | ✅ | 瀑布图 Web 工具 |

## 质量指标

| 指标 | 数值 |
|------|------|
| 代码行数 | ~7,500 |
| 单元测试 | 59 |
| Clippy 警告 | 0 |
| Shell 命令 | 23 |
| Trace span 覆盖 | 引导 + 全部 syscall + 全部 VFS |

## 文档索引

| 文档 | 说明 |
|------|------|
| [README.md](README.md) | 快速开始 |
| [spec.md](spec.md) | 技术规格 |
| [claude.md](claude.md) | AI 自驱动开发规则 |
| [docs/](docs/README.md) | 文档目录索引 |
| [docs/releases/](docs/releases/) | 版本发布说明 |
| [docs/changelogs/](docs/changelogs/) | 开发变更全记录 |
| [docs/dev-logs/](docs/dev-logs/) | AI Coding 任务和数据 |
| [docs/adr/](docs/adr/) | 架构决策记录 |

## 问题追踪
通过 GitHub Issues 管理，使用标签区分：
- `risk` — 风险项
- `decision` — 待决策项
- `bug` — Bug
- `enhancement` — 功能需求
