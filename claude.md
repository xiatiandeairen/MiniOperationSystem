# claude.md — AI 自驱动开发规则

## 自驱动工作循环

```
分析项目 → 找到需求(ROI排序) → 开发需求 → 验证需求
    ↑         ↓ 失败≥3次: 回滚+记录+放弃           ↓
    │         ↓ 成功: commit+push                   ↓
    │     总结本轮不足 → 优化流程 → 同步main → 下一轮
    └──────────────────────────────────────────────┘
```

### 每轮迭代必须回答的问题
1. **方向正确吗?** — 这件事是否让项目离"帮开发者理解OS"更近?
2. **ROI值得吗?** — 投入的时间与可见产出是否匹配?
3. **可回滚吗?** — 每个commit是否独立可回滚，不引入不可逆变更?

### Commit 纪律
- 每个commit可独立回滚，不破坏前后commit
- commit前必须: `cargo build` + `cargo clippy -- -D warnings` + `cargo fmt --check`
- commit message: `<type>(<scope>): <解决了什么问题>`

## 版本发布流程（每版本必须执行）

### 发布前质量门禁
运行 `./scripts/release-check.sh vX.Y.Z`，全部通过才能打 tag：

```
--- Code Quality ---
  ✅ cargo fmt
  ✅ cargo clippy (0 warnings)
  ✅ debug build
  ✅ release build
--- Tests ---
  ✅ unit tests (>=59)
--- Documentation ---
  ✅ release notes (docs/releases/vX.Y.Z.md)
  ✅ changelog (docs/changelogs/vX.Y.Z-changelog.md)
  ✅ tasks (docs/dev-logs/vX.Y.Z-tasks.md)
  ✅ insights (docs/dev-logs/vX.Y.Z-insights.md)
--- Boot Image ---
  ✅ boot-image tool built
  ✅ kernel ELF exists
```

### 发布文档 checklist（每版本 4 个文件）
1. `docs/releases/vX.Y.Z.md` — 精华特性（面向用户）
2. `docs/changelogs/vX.Y.Z-changelog.md` — 全部变更（面向开发者）
3. `docs/dev-logs/vX.Y.Z-tasks.md` — AI Coding 任务 ✅/❌ 清单
4. `docs/dev-logs/vX.Y.Z-insights.md` — 代码量/功能数/ROI/关键学习

### 发布步骤
```bash
./scripts/release-check.sh vX.Y.Z          # 质量门禁
git checkout main && git merge dev-branch   # 合入 main
git tag -a vX.Y.Z -m "vX.Y.Z — Codename"  # 打 tag
git push origin main --tags                 # 推送
gh release create vX.Y.Z --notes-file ...  # 创建 GitHub Release
```

## CI 规则 (GitHub Actions)

每次 push/PR 自动运行:
1. `cargo fmt --check` — 格式
2. `cargo clippy -- -D warnings` — lint（零警告）
3. `cargo build` — debug + release 双构建
4. `cargo test` — 单元测试（≥59 个，回归保护）
5. 项目指标输出到 Summary（源文件数/代码行数/Shell 命令数）

## 文档归档规则

### 每版本必须归档的 4 个文件
| 文件 | 内容 | 模板 |
|------|------|------|
| `releases/vX.Y.Z.md` | 精华特性 + 数字摘要 | 版本概览 → 新增特性 → 数字摘要 |
| `changelogs/vX.Y.Z-changelog.md` | 全部 commit 按类型分组 | feat/fix/test/docs |
| `dev-logs/vX.Y.Z-tasks.md` | 任务 ✅/❌ 清单 + 完成率 | 按 Feature 分组 |
| `dev-logs/vX.Y.Z-insights.md` | 代码变更/ROI/关键学习 | 数据表 + ROI 星级 |

### 问题追踪
通过 GitHub Issues 管理，标签: `risk`, `decision`, `bug`, `enhancement`

## 评分标准

| 维度 | 权重 | 行业优秀 |
|------|------|---------|
| 功能正确性 | 25% | >95% |
| 代码质量 | 20% | CC<10, 0 clippy |
| 测试覆盖 | 15% | >80% |
| 文档完整 | 10% | >90% API doc |
| 架构一致性 | 15% | 0 违反 |
| 交付效率 | 15% | <10% 返工 |

## 沉淀的规则
1. 保持简洁快速节奏，避免引入复杂度
2. 每个功能必须端到端可验证（不只是编译通过）
3. ISR 中绝不能获取 Mutex（trace_event/serial_println 都会死锁）
4. framebuffer 颜色用 BGR 顺序
5. crash 命令不能真正 panic（90% 水位停止）
6. cmd_run 必须释放 VFS 锁后再执行命令（否则 FS 命令死锁）
7. 每次改动调度/中断代码后必须 QEMU GUI 验证 Shell 交互
