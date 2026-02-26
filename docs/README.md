# docs/ 文档目录

## 目录结构

```
docs/
├── README.md                    ← 本文件: 文档索引
├── releases/                    ← 版本发布说明 (精华特性)
│   ├── v0.1.0.md
│   ├── v0.2.0.md
│   └── v0.2.0-roadmap.md
├── changelogs/                  ← 版本开发变更全记录 (编号归档)
│   ├── v0.1.0-changelog.md
│   └── v0.2.0-changelog.md
├── dev-logs/                    ← 版本开发过程记录 (编号归档)
│   ├── v0.1.0-tasks.md          ← AI Coding 任务完成情况
│   ├── v0.1.0-insights.md       ← 开发数据和 ROI 分析
│   ├── v0.2.0-tasks.md
│   └── v0.2.0-insights.md
└── adr/                         ← 架构决策记录 (编号归档)
    ├── 001-use-rust-nightly.md
    ├── 002-bootloader-api-crate.md
    └── 003-multi-crate-workspace.md
```

## 文档编号规则

- **Release Notes**: `releases/vX.Y.Z.md` — 面向用户的精华特性
- **Changelog**: `changelogs/vX.Y.Z-changelog.md` — 全部开发变更，按 commit 粒度
- **Tasks**: `dev-logs/vX.Y.Z-tasks.md` — AI Coding 任务 todolist (✅/❌/⬜)
- **Insights**: `dev-logs/vX.Y.Z-insights.md` — 代码变更量、功能/bug 数、耗时、ROI
- **ADR**: `adr/NNN-title.md` — 架构决策记录

## 风险和待决策项

不在文档中维护，通过 GitHub Issues 管理：
- `label:risk` — 风险项
- `label:decision` — 待决策项
- `label:bug` — Bug
- `label:enhancement` — 功能需求
