# claude.md — AI Coding 协作规则

## 角色定义
- **AI（执行者）**: 按任务文档执行开发，记录过程明细和风险点
- **人类（决策者）**: 提出需求、验收任务、输出改进建议

## 工作流程

### 三文档闭环

```
需求文档 (docs/tasks/TASK-NNN-requirement.md)
    ↓ AI 执行
任务文档 (docs/tasks/TASK-NNN-execution.md)
    ↓ 人类验收
反馈文档 (docs/feedback/TASK-NNN-feedback.md)
    ↓ 沉淀到 claude.md / project.md
```

### 1. 需求阶段
人类在 docs/tasks/ 创建 TASK-NNN-requirement.md，包含：
- 任务目标
- 验收标准
- 优先级和约束

### 2. 执行阶段
AI 创建 docs/tasks/TASK-NNN-execution.md，包含：
- 执行明细（每个步骤做了什么）
- 风险点和决策记录
- 测试结果
- 多维度自评打分

### 3. 反馈阶段
人类验收后，AI 创建 docs/feedback/TASK-NNN-feedback.md，包含：
- 人类建议的原文
- AI 的理解和改进计划
- 沉淀的规则（追加到本文件）

## 评分标准（每轮任务结束时自评）

| 维度 | 权重 | 行业优秀 | 说明 |
|------|------|---------|------|
| 功能正确性 | 25% | >95% | 验收标准满足率 |
| 代码质量 | 20% | CC<10, 0 clippy | 复杂度、lint、命名 |
| 测试覆盖 | 15% | >80% | 关键路径测试覆盖 |
| 文档完整 | 10% | >90% API doc | rustdoc + 注释 |
| 架构一致性 | 15% | 0 违反 | trait 边界、依赖方向 |
| 交付效率 | 15% | <10% 返工 | 一次通过率 |

## 沉淀的协作规则
（随每次反馈迭代追加）
