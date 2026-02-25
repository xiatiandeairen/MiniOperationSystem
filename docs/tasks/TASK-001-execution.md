# TASK-001: 修复代码审查发现的关键缺陷

## 需求来源
行业专家代码审查，识别出 20 项不足，按优先级排列前 5 项执行

## 执行明细

### Fix 1: 上下文切换实际调用 ✅
- **修改**: `crates/kernel/src/main.rs` — `handle_switch()`
- **变更**: 在更新调度器状态后，调用 `switch_context(old_ptr, new_ptr)` 执行真实寄存器切换
- **验证**: QEMU 串口输出 `init process (PID 1) running`，证明 PID 1 的 entry_fn 被执行
- **风险**: `context_ptr()` 返回裸指针时 process table lock 已释放，指针有效性依赖于 static 生命周期

### Fix 2: 单元测试 ✅
- **修改**: `crates/common/src/{id,types,error}.rs`, `crates/trace/src/{span,ringbuffer}.rs`
- **新增**: 46 个 `#[test]` 函数
  - common/id: 7 tests (PID/TraceId/SpanId Display, 生成器单调性)
  - common/types: 8 tests (ColorCode 打包, Priority 排序, OpenFlags 位运算)
  - common/error: 12 tests (From 转换, Display 实现)
  - trace/span: 7 tests (构造, 截断, 字符串视图)
  - trace/ringbuffer: 12 tests (写读, 环绕覆盖, clear, update_span, stats)
- **验证**: `cargo test -p minios-common -p minios-trace --target x86_64-unknown-linux-gnu` — 46 passed

### Fix 3: Trace 集成到引导流程 ✅
- **修改**: `crates/kernel/src/main.rs`
- **变更**: 为 memory_init, filesystem_init, syscall_test, ipc_test, process_init 添加 `trace_span!` 包裹；timer_tick 添加 `trace_event!`
- **验证**: 编译通过，引导流程正常

### Fix 4: 帧缓冲文本渲染 ⏭️ 延后
- **原因**: 需要实现像素级字体渲染（8x16 bitmap font），工作量大且不阻塞其他功能
- **影响**: VGA 输出仅通过串口可见，不影响功能正确性
- **记入**: 后续任务

### Fix 5: 串口 Shell 输入 ✅
- **修改**: `crates/hal/src/serial.rs` 新增 `read_byte()`; `crates/shell/src/shell.rs` 新增 `read_char()` 合并键盘+串口输入
- **验证**: 编译通过，Shell 现在可从 serial 和 keyboard 两个来源读取输入

## 风险点
| ID | 风险 | 影响 | 状态 |
|----|------|------|------|
| R1 | switch_context 在 timer ISR 回调中调用，中断上下文嵌套 | 栈溢出风险 | 🟡 监控 |
| R2 | context_ptr 裸指针在 lock 释放后使用 | UB 风险（实际安全因 static 生命周期） | 🟡 监控 |
| R3 | RingBuffer 测试使用 Box 分配 4096×Span，内存消耗大 | 测试环境正常，CI 需足够内存 | 🟢 已处理 |
| R4 | serial read_byte 在无数据时的 busy-poll | 已用 hlt() 缓解 | 🟢 已处理 |

## 自评打分

| 维度 | 得分 | 满分 | 行业优秀线 | 说明 |
|------|------|------|-----------|------|
| 功能正确性 | 80 | 100 | >95 | 5 项修复完成 4 项，帧缓冲延后 |
| 代码质量 | 75 | 100 | CC<10, 0 clippy | clippy 零警告，但部分函数偏长 |
| 测试覆盖 | 55 | 100 | >80% | 46 tests 覆盖 common+trace，其余 crate 未覆盖 |
| 文档完整 | 70 | 100 | >90% API doc | project.md/claude.md 已建立，API doc 待补 |
| 架构一致性 | 85 | 100 | 0 违反 | 上下文切换已修复，trait 边界完整 |
| 交付效率 | 70 | 100 | <10% 返工 | 帧缓冲延后，串口 Shell 需进一步集成验证 |
| **加权总分** | **73** | **100** | **>85** | |

## 各维度改进方向
- **功能正确性**: 实现帧缓冲文本渲染，使 VGA 输出在图形窗口可见
- **测试覆盖**: 为 scheduler/process/fs/ipc/syscall crate 添加宿主机可运行的单元测试
- **文档完整**: 补充所有 pub API 的 rustdoc，完善 README 的一键体验流程
- **架构一致性**: 将 syscall 改为 int 0x80 陷入式，实现 TraceContext 在 IPC Message 中传播
- **交付效率**: 建立更细粒度的预验收检查，减少延后项
