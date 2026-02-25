# ITER-002: 一键运行 + 帧缓冲文本渲染

## 为什么做
学习者目前无法看到OS输出，也无法一键运行。这是产品体验的0→1问题。

## ROI分析
- 投入: ~90分钟
- 产出: 用户可以 `cargo make run` 一键在QEMU中看到OS输出和Shell
- ROI: 极高 — 解决了最基础的可用性问题

## 任务分解
1. 修复 Makefile.toml run任务（自动build image + QEMU）
2. 实现帧缓冲文本渲染器（8x16 bitmap font → pixel framebuffer）
3. 替换VGA text mode为framebuffer console
4. 验证: QEMU窗口可见OS文本输出

## 验收标准
- [ ] `cargo make run` 一键从编译到QEMU启动
- [ ] QEMU窗口可见 "Hello, MiniOS!" 和引导信息
- [ ] Shell 提示符在QEMU窗口可见

## 约束
- 字体使用最简单的内嵌8x16 bitmap，不引入外部字体文件
- framebuffer console仅实现基础功能：putchar, scroll, newline
- 保持代码简洁，不做过度抽象
