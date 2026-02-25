# TASK-001: 修复代码审查发现的关键缺陷

## 需求来源
行业专家代码审查，20项不足清单

## 执行明细

### Fix 1: 上下文切换实际调用 ✅
- 修改: crates/kernel/src/main.rs handle_switch()
- 变更: 添加 switch_context_asm 调用
- 验证: QEMU串口输出 "init process (PID 1) running"
- 风险: context_ptr 返回裸指针，process table lock已释放

### Fix 2: 单元测试 [状态]
...

### Fix 3: Trace 集成 [状态]
...

### Fix 4: 帧缓冲文本渲染 [状态]
...

### Fix 5: 串口 Shell 输入 [状态]
...

## 风险点
- R1: switch_context 在中断处理器中调用，栈帧嵌套风险
- R2: ...

## 自评打分
(任务完成后填写)
