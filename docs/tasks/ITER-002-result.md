# ITER-002 Result: 一键运行 + 帧缓冲文本渲染

## 成果

### 完成项
1. **一键运行**: `cargo make run` 自动编译(release) → 构建boot image → 启动QEMU
2. **帧缓冲文本渲染**: 实现8×16 bitmap字体渲染器，在QEMU图形窗口显示文本
3. **Shell可见**: QEMU窗口显示 "Hello, MiniOS!" + "Boot successful" + Shell提示符 "MiniOS $"

### 关键文件
- `crates/hal/src/framebuffer.rs` — 651行，完整的像素帧缓冲文本控制台
- `Makefile.toml` — 重构run/build-release/image/boot-img任务链
- `crates/kernel/src/main.rs` — framebuffer初始化集成
- `crates/hal/src/vga.rs` — println!路由修复（framebuffer优先）

### 技术决策
- **字体选择**: 内嵌CP437 8×16 bitmap (95 glyphs, ASCII 0x20-0x7E)，无外部依赖
- **framebuffer获取**: 通过raw pointer从BootInfo提取，避免与memory::init的借用冲突
- **println!路由**: framebuffer可用时只写framebuffer，避免VGA text mode的死锁

## 风险点
| ID | 风险 | 处置 |
|----|------|------|
| R1 | framebuffer slice通过raw pointer构造`&'static mut`，有UB风险 | 接受: bootloader保证内存有效且不重叠 |
| R2 | scroll_up用copy_within性能不高 | 接受: 160×45字符的控制台足够快 |
| R3 | 只支持ASCII 0x20-0x7E | 接受: OS学习项目不需要Unicode |

## 自评打分

| 维度 | 得分 | 说明 |
|------|------|------|
| 功能正确性 | 95 | 3/3验收标准满足: 一键run ✅, 图形输出可见 ✅, Shell提示符 ✅ |
| 代码质量 | 80 | framebuffer.rs 651行偏长，font data占大部分 |
| 测试覆盖 | 40 | 无framebuffer单元测试(需hardware mock) |
| 文档完整 | 75 | 函数有rustdoc，但framebuffer module doc待补 |
| 架构一致性 | 90 | framebuffer作为HAL模块，不破坏分层 |
| 交付效率 | 95 | 0返工，2个commit完成全部功能 |
| **加权总分** | **80** | 上轮73→本轮80, +7 |

## 流程反思
- **做对了**: ROI排序正确，帧缓冲是最高可见价值
- **可改进**: framebuffer.rs应该拆分(font data独立文件)
- **下轮优先**: Shell交互（键盘输入在QEMU图形窗口可用）
