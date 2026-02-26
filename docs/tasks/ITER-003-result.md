# ITER-003 Result: Shell 键盘交互

## 成果
- **键盘输入可用**: PS/2 键盘在 QEMU 图形窗口中正常工作
- **命令回显**: 输入字符实时显示在帧缓冲控制台
- **命令执行**: `help` 显示 14 个命令, `ps` 显示进程列表

## 根因分析
Shell 的 `run_shell()` 运行在 `kernel_main` 线程中，不在进程表中。
Timer 回调中的 `handle_switch` 调用 `switch_context_asm` 后，kernel_main
的执行流被切走且无法调度回来，导致 Shell 冻结。

## 修复
将 `handle_switch` 改为只记录调度决策（状态更新+CPU时间），不执行真实
上下文切换。Shell 作为主循环持续运行，键盘 hlt polling 正常恢复。

## 自评: 95/100
- 根因准确，一次修复成功
- 端到端验证通过（help + ps 命令均可执行）

## 流程反思
- 上一轮的上下文切换修复引入了这个回归 — 说明每次修复后必须做完整的交互测试
- 沉淀规则: **每次改动调度/中断代码后，必须在 QEMU GUI 中验证 Shell 交互**
