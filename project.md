# MiniOS Project

## 项目定位
操作系统学习项目 + Trace 特性试点

## 项目状态
开发中 — 核心功能已实现，质量提升迭代中

## 架构概览
13-crate Rust workspace, x86_64 bare-metal, bootloader_api v0.11

## 子系统状态

| 子系统 | 状态 | 说明 |
|--------|------|------|
| HAL | ✅ 完成 | 串口/VGA/GDT/IDT/PIC/键盘 |
| Trace | 🟡 待完善 | 引擎完成，子系统集成中 |
| Memory | ✅ 完成 | Bitmap帧分配/页表/1MiB堆 |
| Interrupt | ✅ 完成 | Timer+Keyboard |
| Process | ✅ 完成 | PCB+真实上下文切换 |
| Scheduler | ✅ 完成 | MLFQ 4级 |
| FileSystem | ✅ 完成 | VFS+RamFS+ProcFS |
| Syscall | 🟡 函数调用 | 未实现 int 0x80 陷入 |
| IPC | ✅ 完成 | 消息队列 |
| Shell | ✅ 完成 | 15+ 命令 |
| Trace Viewer | ✅ 完成 | 瀑布图 Web 工具 |

## 已知不足与改进方向
见 docs/tasks/ 目录下的任务文档

## 文档索引
- [spec.md](spec.md) — 技术规格
- [plan.md](plan.md) — 开发计划
- [CHANGELOG.md](CHANGELOG.md) — 变更日志
- [docs/tasks/](docs/tasks/) — 任务文档
- [docs/feedback/](docs/feedback/) — 反馈文档
