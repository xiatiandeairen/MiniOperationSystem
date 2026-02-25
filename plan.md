# MiniOS 渐进式开发计划

> **关联规格**: [spec.md](./spec.md)
> **检查点数**: 10
> **任务总数**: ~500
> **原则**: 高质量 · 渐进式 · 少决策 · Clean Code

---

## 第一部分：Vibe Coding 自工作规则

### 1.1 核心工作循环

每个任务遵循以下不可跳过的闭环流程：

```
┌─────────────────────────────────────────────────────────────────┐
│                     单任务工作闭环                                │
│                                                                 │
│  ① 预检查 ──► ② 拆解目标 ──► ③ 开发实现 ──► ④ 自检验收         │
│      │                           │               │              │
│      │                           ▼               ▼              │
│      │                      满足commit?      验收通过?           │
│      │                       │    │           │    │             │
│      │                      Yes   No         Yes   No           │
│      │                       │    │           │    │             │
│      │                       ▼    └──►继续    ▼    └──►修复回③   │
│      │                    git commit       ⑤ 代码审查             │
│      │                                       │                   │
│      │                                    通过?                  │
│      │                                    │    │                 │
│      │                                   Yes   No               │
│      │                                    │    └──►重构回③       │
│      ▼                                    ▼                     │
│  代码健康检查                           任务完成 ──► 下一任务      │
│  (是否需要重构?)                                                 │
│   │        │                                                    │
│  Yes       No ──► 进入②                                         │
│   │                                                             │
│   └──► 重构任务 (单独commit)                                     │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 预检查规则（每个任务开始前必须执行）

```
预检查清单:
├─ 1. cargo build 是否通过？不通过则先修复
├─ 2. cargo test 是否通过？不通过则先修复
├─ 3. cargo clippy 是否通过？不通过则先修复
├─ 4. 上一个任务的验收标准是否全部满足？
├─ 5. 结构性健康检查（任一不满足 → 先重构再开始新任务）
│     ├─ 函数超过 50 行？→ 拆分
│     ├─ 重复代码 > 2 处？→ 抽取公共模块
│     ├─ 模块文件超过 500 行？→ 拆分子模块
│     ├─ TODO/FIXME 累计 > 5 个？→ 清理
│     ├─ 公开接口缺少文档？→ 补充
│     └─ 当前文件 Hotspot RiskScore > 0.6？→ 按 §1.8 执行 P0 重构
├─ 6. 命名可读性检查
│     ├─ 变量名是否回答"这是什么"？
│     │     ✗ data, info, temp, result, ret
│     │     ✓ free_frame_count, keyboard_scancode, trace_ring_buffer
│     ├─ 函数名是否回答"做什么"？
│     │     ✗ process(), handle(), run(), do_work()
│     │     ✓ allocate_physical_frame(), dispatch_syscall(), parse_shell_command()
│     ├─ Bool 变量/函数是否读作自然语言断言？
│     │     ✗ flag, status, check
│     │     ✓ is_page_mapped, has_pending_interrupt, should_reschedule
│     ├─ 类型名是否表达"它是什么角色"？
│     │     ✗ Manager2, DataHolder, Helper
│     │     ✓ BitmapFrameAllocator, MlfqScheduler, VfsPathResolver
│     └─ 违反 → 重命名后再继续
├─ 7. 依赖关系检查（单向流原则）
│     ├─ 模块间依赖是否单向？（上层 → 下层，禁止反向或循环）
│     │     Shell → Syscall → Process/FS → Memory → HAL（唯一合法方向）
│     ├─ 函数是否只依赖参数和显式注入的服务？（禁止隐式全局状态偷取）
│     ├─ 新增 use/import 是否引入了不该存在的跨层依赖？
│     └─ 违反 → 通过 trait 抽象或参数注入消除非法依赖
├─ 8. 函数职责检查（单一职责原则）
│     ├─ 函数是否只做一件事？能否用一句话描述其职责且不用"和"/"或"？
│     │     ✗ "分配帧并映射页表并更新统计"
│     │     ✓ "从空闲位图中分配一个物理帧"
│     ├─ 函数是否混合了策略与机制？
│     │     策略 = 决定做什么（如：选择下一个调度的进程）
│     │     机制 = 怎么做（如：保存/恢复 CPU 上下文）
│     │     策略和机制必须分离到不同函数
│     ├─ 函数是否有隐式副作用？（修改全局状态、I/O 操作）
│     │     所有副作用必须在函数签名或名称中显式体现
│     │     ✗ fn get_frame() 内部修改全局计数器
│     │     ✓ fn allocate_frame() 名称暗示会改变状态
│     └─ 违反 → 拆分为纯计算函数 + 副作用函数
├─ 9. 注释质量检查
│     ├─ 注释是否解释"为什么这么做"而非"做了什么"？
│     │     ✗ // 遍历位图查找空闲帧
│     │     ✓ // 使用线性扫描而非 buddy 算法，因为帧数 <1M 时性能差异可忽略
│     ├─ 代码本身能表达的信息是否被注释重复了？→ 删除冗余注释
│     ├─ 存在反直觉的实现是否有注释解释原因？
│     │     ✓ // PIC 重映射到 32-47 是因为 0-31 被 CPU 异常占用
│     ├─ unsafe 块是否有 SAFETY 注释说明为什么安全？
│     └─ 违反 → 修正注释再继续
├─ 10. 状态变迁检查
│     ├─ 有状态的对象是否有明确的状态枚举？（禁止用 bool 组合表示状态）
│     │     ✗ is_running: bool + is_blocked: bool
│     │     ✓ state: ProcessState { Created, Ready, Running, Blocked, Terminated }
│     ├─ 状态转换是否在单一位置管理？（禁止散落在多处直接修改状态字段）
│     ├─ 非法状态转换是否被类型系统或运行时检查拦截？
│     └─ 违反 → 引入状态机模式重构
└─ 11. 重构需求存在？→ 先完成重构 commit，再开始新任务
```

### 1.3 Commit 规则

**原则**: 一个 commit = 一个原子性变更（一个 feature 或一个 bugfix）

**Commit 消息格式** (Conventional Commits):
```
<type>(<scope>): <简短描述>
```

仅此一行，不加 body 和 footer。简短描述必须说明**解决了什么问题 / 达成了什么能力**，而非描述做了什么操作。

**描述撰写原则 — 说"解决了什么"，不说"做了什么"**:

| ✗ 错误（描述操作） | ✓ 正确（描述解决的问题） |
|---|---|
| `feat(memory): implement bitmap frame allocator` | `feat(memory): enable physical frame allocation via bitmap` |
| `fix(scheduler): change priority comparison logic` | `fix(scheduler): prevent low-priority tasks from starving` |
| `refactor(hal): extract port I/O into separate module` | `refactor(hal): isolate port I/O for independent testability` |
| `test(memory): add frame allocator unit tests` | `test(memory): verify frame alloc/dealloc correctness` |
| `docs(trace): add rustdoc for Tracer trait` | `docs(trace): clarify Tracer trait contract for implementors` |
| `chore: update rust-toolchain` | `chore: lock nightly toolchain for reproducible builds` |

**Type 枚举**:
| Type | 说明 |
|------|------|
| `feat` | 新增能力 |
| `fix` | 修复缺陷 |
| `refactor` | 改善结构（不改变行为） |
| `test` | 补充/修正测试 |
| `docs` | 文档变更 |
| `chore` | 构建/工具链/CI 变更 |

**Commit 自检清单**:
- [ ] 本次 commit 只包含一个功能/修复
- [ ] `cargo build` 通过
- [ ] `cargo test` 通过（如有测试）
- [ ] `cargo clippy` 无 warning
- [ ] `cargo fmt --check` 通过
- [ ] 新增公开接口有 rustdoc 注释
- [ ] 无遗留的调试代码（`dbg!`, 临时 `println!` 等）
- [ ] commit 消息描述的是"解决了什么"而非"做了什么"

### 1.4 卡住时的处理策略

```
卡住判定: 在同一问题上尝试 > 3 次不同方案均失败

处理流程:
├─ Level 1 (自行解决):
│   ├─ 重新阅读 spec.md 相关章节
│   ├─ 检查参考资料 (os.phil-opp.com, OSDev Wiki)
│   ├─ 简化实现（先用最简方案，标记 TODO）
│   └─ 跳过该任务，记录到 RISKS.md，继续下一个任务
│
├─ Level 2 (降级实现):
│   ├─ 用 stub/mock 替代完整实现
│   ├─ 在代码中标记: // STUB: [reason] - needs real implementation
│   └─ 在 DECISIONS.md 记录降级决策和影响范围
│
└─ Level 3 (标记人工介入):
    ├─ 在 RISKS.md 记录问题详情
    ├─ 标记为 🔴 BLOCKED
    ├─ 继续开发不依赖此功能的其他任务
    └─ 在检查点汇报时提出
```

### 1.5 代码质量标准 (Clean Code)

**强制规则**:

| 规则 | 阈值 | 违反时操作 |
|------|------|-----------|
| 单个函数行数 | ≤ 50 行 | 拆分为子函数 |
| 单个文件行数 | ≤ 500 行 | 拆分为子模块 |
| 函数参数个数 | ≤ 5 个 | 使用结构体封装 |
| 嵌套深度 | ≤ 4 层 | 提前返回 / 提取函数 |
| 重复代码 | ≤ 2 处 | 第 3 次出现时抽取 |
| `unsafe` 代码 | 每处必须有 SAFETY 注释 | 无注释不提交 |
| 公开 API | 必须有 rustdoc | 无文档不提交 |
| 命名 | 必须自描述 | 禁止单字母变量（循环变量除外） |

**重构触发条件**（任一满足则在新任务开始前先重构）:

1. 新增功能导致现有函数超过 50 行
2. 发现第 3 处重复代码
3. 模块内部出现循环依赖
4. Trait 接口需要修改（影响多个实现）
5. 测试覆盖率明显不足（新增逻辑无测试）
6. **散弹枪手术**: 新增需求导致改动旧代码处超过 2 处 → 说明职责边界划分有问题，立刻重构将变更收敛到单一模块
7. **兼容代码气味**: 需要添加 `if`/`match` 分支做兼容或特殊处理 → 立刻重构为可扩展结构（trait / 策略模式 / 枚举 dispatch），使后续需求可通过新增代码而非修改旧代码来满足
8. **上下文碎片化**: 修改代码后，理解某个行为需要跳转 > 2 个文件才能拼凑完整逻辑 → 立刻重构，将相关逻辑内聚到同一模块，保证局部可推理性（读一个函数/模块就能理解完整行为）
9. **隐式耦合**: 修改 A 模块后 B 模块意外失败 → 说明存在隐式依赖，立刻通过显式接口（trait 参数/返回值）替代隐式共享状态

### 1.6 测试策略

```
每个任务的测试要求:
├─ 新增 Trait → 至少 1 个 Mock 实现 + 1 个测试
├─ 新增数据结构 → 构造/基本操作测试
├─ 新增算法逻辑 → 正常路径 + 边界条件 + 错误路径测试
├─ 新增系统集成点 → QEMU 中运行验证
└─ Bug 修复 → 先写复现测试，再修复，确保测试通过
```

### 1.7 检查点汇报模板

每个检查点完成后，生成以下汇报：

```markdown
## 检查点 N 完成报告

### 基本信息
- 检查点: CPn - [名称]
- 计划任务数: N
- 完成任务数: N
- 跳过任务数: N (原因: ...)
- 总 commit 数: N

### 验收标准达成情况
- [x] 标准 1
- [x] 标准 2
- [ ] 标准 3 (原因: ...)

### 代码质量指标
- cargo build: ✅
- cargo test: ✅ (N tests passed)
- cargo clippy: ✅ (0 warnings)
- cargo fmt: ✅

### 风险与问题
- 🟡 [问题描述] - [影响] - [建议处理方式]
- 🔴 [阻塞问题] - [需要人工决策]

### 技术债务
- TODO 数量: N
- STUB 数量: N

### 下一检查点准备状态
- [x] 依赖的接口已定义
- [x] 测试基础设施就绪

### Hotspot 风险热点
- 🔴 P0 文件: [列表] (满足 4/4 条件)
- 🟡 P1 文件: [列表] (满足 3/4 条件)
- ⚪ P2 文件: [列表] (满足 2/4 条件)
```

### 1.8 Hotspot 风险评分模型

代码质量不能仅靠编写时的主观判断，必须用**数据驱动的结构风险评估**量化识别高风险文件。每个检查点完成时必须运行 Hotspot 分析，结果记入检查点报告。

#### 1.8.1 四个评估维度

##### 维度 ①：复杂度 (Complexity)

| 指标 | 计算方式 | 警戒阈值 |
|------|---------|---------|
| 圈复杂度 CC_total | 文件内所有函数 CC 求和 | > 50 |
| 最大单函数 CC | 单个函数的 CC | > 15 |
| 最大嵌套深度 | 代码块嵌套层数 | > 4 |
| 文件行数 LOC | 非注释代码行 | > 500 |

**标准化得分** (归一化到 0~1):

```
ComplexityScore =
    0.4 × min(1, CC_total / 50) +
    0.3 × min(1, MaxFnCC / 15) +
    0.3 × min(1, MaxNesting / 4)
```

**Rust 项目测量方法**:
- 使用 `rust-code-analysis` 工具计算 CC
- 或手工统计：每个 `if` / `match arm` / `while` / `for` / `&&` / `||` 各计 +1

##### 维度 ②：高频修改 (Change Frequency)

| 指标 | 计算方式 | 警戒阈值 |
|------|---------|---------|
| 近期提交次数 | `git log --since="3 months" --oneline -- <file> \| wc -l` | > 10 |
| 修改作者数 | `git shortlog -s -- <file> \| wc -l` | > 3 |
| 行变更总数 | 累计 insertions + deletions | > 300 |

**标准化得分**:

```
ChangeScore =
    0.6 × min(1, CommitCount / 10) +
    0.4 × min(1, AuthorCount / 3)
```

**注意**: 项目初期（CP1-CP3）所有文件提交次数都较高属于正常现象，Hotspot 分析从 CP4 开始才具有参考意义。

##### 维度 ③：缺陷密度 (Defect Density)

| 指标 | 计算方式 | 警戒阈值 |
|------|---------|---------|
| Bug 修复次数 | commit message 中含 `fix` 的提交数 | > 5 / 3个月 |
| 缺陷密度 | Bug修复次数 / LOC × 1000 | > 20 |
| 回滚次数 | revert 次数 | > 2 |

**标准化得分**:

```
DefectScore = min(1, BugFixCount / 5)
```

##### 维度 ④：测试覆盖 (Test Coverage)

| 指标 | 警戒阈值 |
|------|---------|
| 行覆盖率 | < 60% |
| 关键函数无测试 | 是 |
| unsafe 块无测试 | 是 |

**标准化得分**:

```
CoverageScore = 1 - CoverageRate
```

例：覆盖率 80% → Score = 0.2（低风险），覆盖率 40% → Score = 0.6（高风险）

**Rust no_std 测量方法**:
- 可单元测试的模块（pure logic）：使用 `cargo tarpaulin` 或手工统计
- 不可测模块（硬件依赖）：统计是否有 Mock trait + 对应测试
- 无法自动测量时，按以下规则赋分：
  - 有完整测试（正常+边界+错误） → 0.1
  - 有基本测试 → 0.3
  - 仅有 smoke test → 0.6
  - 无测试 → 1.0

#### 1.8.2 综合风险得分

##### 加权线性模型（常规使用）

```
RiskScore =
    0.30 × ComplexityScore +
    0.30 × ChangeScore +
    0.25 × DefectScore +
    0.15 × CoverageScore
```

| 风险等级 | RiskScore 范围 | 操作 |
|---------|---------------|------|
| 🟢 低风险 | 0 ~ 0.3 | 无需特殊关注 |
| 🟡 中风险 | 0.3 ~ 0.6 | 下个检查点前安排重构 |
| 🔴 高风险 | 0.6 ~ 1.0 | 立刻停止新功能，优先重构 |

##### 乘法模型（识别"四高"极端热点）

```
MultiplicativeRisk = ComplexityScore × ChangeScore × DefectScore × CoverageScore
```

任何一项接近 0 → 总风险下降；四项都高 → 指数级上升。`MultiplicativeRisk > 0.3` 即为严重热点。

#### 1.8.3 硬性判定规则（无需计算版）

**判定为 P0 高风险热点文件**的条件——满足以下 4 条中 ≥ 3 条：

```
IF:
    CC_total > 50
    AND CommitCount(3个月) > 10
    AND BugFixCount(3个月) > 5
    AND TestCoverage < 60%
THEN:
    标记为 🔴 P0 重构对象 → 立刻重构，阻塞新功能
```

**判定为 P1 中风险热点**——满足 2 条：

```
标记为 🟡 P1 → 当前检查点结束前必须重构
```

**判定为 P2 低风险关注**——满足 1 条：

```
标记为 ⚪ P2 → 记录到风险登记簿，下个检查点复查
```

#### 1.8.4 Hotspot 分析执行时机

| 时机 | 动作 |
|------|------|
| 每个检查点完成时 | **必须执行**完整 Hotspot 分析，结果写入检查点报告 |
| 每次重构前 | 检查目标文件的 RiskScore，优先处理得分最高的 |
| 发现连续 fix commit 时 | 对涉及文件立即计算 DefectScore |
| 代码审查发现高嵌套/长函数 | 对该文件计算 ComplexityScore |

#### 1.8.5 Hotspot 分析脚本

在每个检查点完成时执行以下 Git 分析（自动化）：

```bash
#!/bin/bash
# scripts/hotspot-analysis.sh
# 输出近 3 个月各文件的修改频率和 fix 次数

echo "=== 修改频率 Top 20 ==="
git log --since="3 months" --name-only --pretty=format: -- 'crates/**/*.rs' \
  | sort | uniq -c | sort -rn | head -20

echo ""
echo "=== Bug 修复热点 ==="
git log --since="3 months" --name-only --pretty=format: --grep="^fix" -- 'crates/**/*.rs' \
  | sort | uniq -c | sort -rn | head -20

echo ""
echo "=== 文件行数 Top 20 ==="
find crates -name '*.rs' -exec wc -l {} + | sort -rn | head -20

echo ""
echo "=== 多作者文件 ==="
for f in $(find crates -name '*.rs'); do
  authors=$(git shortlog -s -- "$f" 2>/dev/null | wc -l)
  if [ "$authors" -gt 2 ]; then
    echo "$authors authors: $f"
  fi
done
```

#### 1.8.6 工业级 Hotspot 交叉分析

当项目进入 CP4+ 阶段后，使用 Google/Microsoft 常用的交叉排序法：

```
1. 按修改频率排序 → 取 Top 20% 文件
2. 按复杂度排序   → 取 Top 20% 文件
3. 取交集         → 这些就是真正的 Hotspot
4. 交集内按缺陷数排序 → 得到重构优先级
```

这种方法比单一维度阈值判定更精确：
- 高复杂度但从不修改 → 不是当前风险（稳定的复杂代码）
- 高频修改但很简单 → 不是结构风险（活跃但健康的代码）
- **高复杂度 + 高频修改** → 真正的结构风险热点

---

## 第二部分：文档归档机制

### 2.1 文档结构

```
docs/
├── adr/                          # 架构决策记录
│   ├── 000-template.md           # ADR 模板
│   ├── 001-use-rust-nightly.md   # 示例
│   └── ...
├── progress/                     # 进度记录
│   ├── cp01-report.md            # 检查点 1 完成报告
│   ├── cp02-report.md
│   └── ...
├── risks/                        # 风险登记
│   └── risk-register.md          # 风险登记簿
└── decisions/                    # 待人工决策项
    └── pending-decisions.md      # 待决策清单
```

### 2.2 架构决策记录 (ADR) 模板

```markdown
# ADR-NNN: [标题]

## 状态
Proposed | Accepted | Deprecated | Superseded by ADR-XXX

## 背景
[为什么需要做这个决策]

## 决策
[做了什么决策]

## 备选方案
1. [方案A] - 优点/缺点
2. [方案B] - 优点/缺点

## 影响
[这个决策的影响范围和后果]
```

### 2.3 风险登记簿格式

```markdown
| ID | 风险描述 | 概率 | 影响 | 状态 | 缓解措施 | 关联任务 |
|----|---------|------|------|------|---------|---------|
| R01 | ... | 高/中/低 | 高/中/低 | 🟢开放/🟡监控/🔴阻塞/⚪已关闭 | ... | CPn-Tnn |
```

### 2.4 待人工决策清单格式

```markdown
| ID | 决策问题 | 上下文 | 默认选择 | 影响范围 | 紧急度 | 状态 |
|----|---------|--------|---------|---------|--------|------|
| D01 | ... | ... | [AI 推荐的默认选择] | ... | 高/中/低 | 待定/已决策 |
```

**规则**: AI 在开发中遇到需要决策时：
1. 如果影响范围小（单个模块内部）→ 自行决策，记录到 ADR
2. 如果影响范围大（跨模块接口）→ 选择最保守方案，记录到待决策清单
3. 如果是阻塞性决策 → 标记 🔴，用降级方案继续，检查点时汇报

### 2.5 CHANGELOG 格式

```markdown
# Changelog

## [CP1] - 项目脚手架与构建系统
### Added
- Cargo workspace with 12 crates
- rust-toolchain.toml (nightly)
- Makefile.toml (cargo-make tasks)
### Changed
- (none)
### Fixed
- (none)
```

---

## 第三部分：检查点与任务详细计划

---

# CP1: 项目脚手架与构建系统

> **目标**: 建立完整的 Rust workspace 骨架，配置工具链和构建系统，确保空项目可编译
> **预计任务数**: 48
> **前置条件**: 无

## CP1 验收标准

- [ ] **AC-1.1**: `cargo build --target x86_64-unknown-none` 编译通过（零错误零警告）
- [ ] **AC-1.2**: `cargo test` 在宿主机上通过（针对非 `no_std` 可测试部分）
- [ ] **AC-1.3**: `cargo clippy --all-targets` 零警告
- [ ] **AC-1.4**: `cargo fmt --all --check` 通过
- [ ] **AC-1.5**: 所有 12 个 crate 骨架存在且可编译
- [ ] **AC-1.6**: `rust-toolchain.toml` 正确锁定 nightly 版本
- [ ] **AC-1.7**: `Makefile.toml` 中 build/test/clippy/fmt 四个任务可用
- [ ] **AC-1.8**: `.github/workflows/ci.yml` CI 配置正确（可不运行但语法正确）
- [ ] **AC-1.9**: 项目 README 包含构建和运行说明
- [ ] **AC-1.10**: 文档归档目录结构已建立

## CP1 任务列表

### CP1-T01: 初始化 Cargo workspace 根配置
- **目标**: 创建 workspace 根 `Cargo.toml`，定义所有 member crate
- **开发任务**:
  1. 创建 `Cargo.toml`，设置 `[workspace]` 和 `members` 列表
  2. 设置 workspace 级别的 `[profile.dev]` 和 `[profile.release]`
  3. 设置 workspace 级别的 `[workspace.dependencies]` 共享依赖
- **Commit**: `chore: establish multi-crate workspace for modular kernel`
- **验收标准**:
  - [ ] `Cargo.toml` 中 members 包含所有 12 个 crate 路径
  - [ ] workspace dependencies 定义了 `spin`, `bitflags`, `volatile` 等共享依赖
- **代码审查**: 格式规范，无多余注释

### CP1-T02: 配置 Rust 工具链
- **目标**: 锁定 nightly 工具链版本，确保可复现构建
- **开发任务**:
  1. 创建 `rust-toolchain.toml`
  2. 指定 channel = "nightly"
  3. 添加 components: rust-src, llvm-tools-preview, rustfmt, clippy
  4. 添加 targets: x86_64-unknown-none
- **Commit**: `chore: lock nightly toolchain for reproducible bare-metal builds`
- **验收标准**:
  - [ ] `rustup show` 显示正确的 nightly 版本
  - [ ] `rustc --version` 输出 nightly
  - [ ] `x86_64-unknown-none` target 已安装
- **代码审查**: 版本号合理，components 完整

### CP1-T03: 创建自定义编译目标 (target spec)
- **目标**: 定义 x86_64 裸机编译目标的 JSON 规格（如需自定义；或直接使用内置 target）
- **开发任务**:
  1. 评估是否需要自定义 target JSON（内置 `x86_64-unknown-none` 可能已足够）
  2. 如需自定义：创建 `x86_64-minios.json`
  3. 在 `.cargo/config.toml` 中设置默认 target
- **Commit**: `chore: enable bare-metal compilation for x86_64`
- **验收标准**:
  - [ ] `.cargo/config.toml` 存在且 target 配置正确
  - [ ] 裸机编译不报 target 相关错误
- **代码审查**: target spec 参数合理（disable-redzone, panic=abort 等）

### CP1-T04: 创建 `crates/hal` crate 骨架
- **目标**: 创建 HAL crate 的目录和最小可编译代码
- **开发任务**:
  1. 创建 `crates/hal/Cargo.toml`（`name = "minios-hal"`, `no_std`）
  2. 创建 `crates/hal/src/lib.rs`（`#![no_std]`，空模块声明）
  3. 声明将要包含的子模块（注释形式）
- **Commit**: `chore(hal): establish HAL crate for hardware isolation`
- **验收标准**:
  - [ ] `cargo build -p minios-hal --target x86_64-unknown-none` 通过
  - [ ] lib.rs 包含 `#![no_std]` 声明
- **代码审查**: Cargo.toml metadata 完整（description, edition）

### CP1-T05: 创建 `crates/trace` crate 骨架
- **目标**: 创建 Trace 引擎 crate 的目录和最小可编译代码
- **开发任务**:
  1. 创建 `crates/trace/Cargo.toml`
  2. 创建 `crates/trace/src/lib.rs`（`#![no_std]`）
  3. 添加对 `minios-hal` 的依赖（workspace dependency）
- **Commit**: `chore(trace): establish trace crate for observability`
- **验收标准**:
  - [ ] `cargo build -p minios-trace --target x86_64-unknown-none` 通过
  - [ ] 正确依赖 `minios-hal`
- **代码审查**: 依赖方向正确（trace → hal）

### CP1-T06: 创建 `crates/trace-macros` crate 骨架
- **目标**: 创建过程宏 crate（注意：proc-macro crate 运行在宿主机，非 no_std）
- **开发任务**:
  1. 创建 `crates/trace-macros/Cargo.toml`（`proc-macro = true`）
  2. 创建 `crates/trace-macros/src/lib.rs`
  3. 添加 `syn`, `quote`, `proc-macro2` 依赖
- **Commit**: `chore(trace-macros): establish proc-macro crate for trace annotations`
- **验收标准**:
  - [ ] `cargo build -p minios-trace-macros` 通过（注意不需要 --target）
  - [ ] Cargo.toml 中 `[lib] proc-macro = true` 设置正确
- **代码审查**: proc-macro 依赖版本合理

### CP1-T07: 创建 `crates/memory` crate 骨架
- **目标**: 创建内存管理 crate 骨架
- **开发任务**:
  1. 创建 `crates/memory/Cargo.toml`
  2. 创建 `crates/memory/src/lib.rs`（`#![no_std]`）
  3. 添加 workspace dependencies
- **Commit**: `chore(memory): establish memory management crate`
- **验收标准**:
  - [ ] crate 可编译
  - [ ] 依赖声明正确
- **代码审查**: feature flags 定义合理

### CP1-T08: 创建 `crates/interrupt` crate 骨架
- **目标**: 创建中断处理 crate 骨架
- **开发任务**:
  1. 创建目录和文件
  2. 声明 `#![no_std]`
- **Commit**: `chore(interrupt): establish interrupt handling crate`
- **验收标准**:
  - [ ] crate 可编译
- **代码审查**: 依赖方向正确

### CP1-T09: 创建 `crates/process` crate 骨架
- **目标**: 创建进程管理 crate 骨架
- **开发任务**: 同上模式
- **Commit**: `chore(process): establish process management crate`
- **验收标准**:
  - [ ] crate 可编译
- **代码审查**: 通过

### CP1-T10: 创建 `crates/scheduler` crate 骨架
- **目标**: 创建调度器 crate 骨架
- **Commit**: `chore(scheduler): establish task scheduling crate`
- **验收标准**:
  - [ ] crate 可编译

### CP1-T11: 创建 `crates/fs` crate 骨架
- **目标**: 创建文件系统 crate 骨架
- **Commit**: `chore(fs): establish filesystem crate`
- **验收标准**:
  - [ ] crate 可编译

### CP1-T12: 创建 `crates/ipc` crate 骨架
- **目标**: 创建 IPC crate 骨架
- **Commit**: `chore(ipc): establish inter-process communication crate`
- **验收标准**:
  - [ ] crate 可编译

### CP1-T13: 创建 `crates/syscall` crate 骨架
- **目标**: 创建系统调用 crate 骨架
- **Commit**: `chore(syscall): establish system call interface crate`
- **验收标准**:
  - [ ] crate 可编译

### CP1-T14: 创建 `crates/shell` crate 骨架
- **目标**: 创建 Shell crate 骨架
- **Commit**: `chore(shell): establish interactive terminal crate`
- **验收标准**:
  - [ ] crate 可编译

### CP1-T15: 创建 `crates/kernel` crate 骨架
- **目标**: 创建内核集成 crate（最终可执行 crate）
- **开发任务**:
  1. 创建 `crates/kernel/Cargo.toml`
  2. 创建 `crates/kernel/src/main.rs`（`#![no_std]`, `#![no_main]`）
  3. 添加 panic_handler 和 _start 入口（空实现）
  4. 依赖所有其他 crate
- **Commit**: `chore(kernel): establish bootable kernel entry crate`
- **验收标准**:
  - [ ] `cargo build -p minios-kernel --target x86_64-unknown-none` 通过
  - [ ] 包含 `#![no_std]` 和 `#![no_main]`
  - [ ] panic_handler 已定义
- **代码审查**: 入口函数签名正确

### CP1-T16: 定义 workspace 共享类型 crate
- **目标**: 创建 `crates/common` 存放跨 crate 共享的基础类型
- **开发任务**:
  1. 创建 `crates/common/Cargo.toml`
  2. 定义 `Pid`, `FileDescriptor`, `InodeId` 等基础类型
  3. 定义公共错误枚举 `KernelError`
- **Commit**: `feat(common): enable cross-crate type sharing`
- **验收标准**:
  - [ ] 基础 ID 类型定义（Pid, FileDescriptor, QueueId 等）
  - [ ] 公共错误类型定义
  - [ ] 所有类型实现 `Debug`, `Clone`, `Copy`
- **代码审查**: 类型设计简洁，无冗余

### CP1-T17: 配置 workspace 级别的依赖
- **目标**: 在 workspace Cargo.toml 中统一管理外部依赖版本
- **开发任务**:
  1. 添加 `x86_64`, `bootloader`, `spin`, `volatile`, `uart_16550`, `bitflags`, `linked_list_allocator` 等
  2. 各 crate 使用 `workspace = true` 引用
- **Commit**: `chore: prevent dependency version drift across crates`
- **验收标准**:
  - [ ] 所有外部依赖在 workspace root 统一声明
  - [ ] 各 crate 使用 `dependency.workspace = true`
- **代码审查**: 依赖版本合理，无冲突

### CP1-T18: 配置 `.cargo/config.toml`
- **目标**: 设置 cargo 构建配置（默认 target、链接器、rustflags）
- **开发任务**:
  1. 设置 `[build] target = "x86_64-unknown-none"`
  2. 设置 runner（QEMU 命令）
  3. 设置 rustflags（如需要）
- **Commit**: `chore: default cargo build to bare-metal target`
- **验收标准**:
  - [ ] `cargo build` 默认编译为 x86_64-unknown-none
  - [ ] config.toml 语法正确
- **代码审查**: 配置项合理

### CP1-T19: 安装 cargo-make 并创建 Makefile.toml
- **目标**: 建立任务编排系统
- **开发任务**:
  1. 创建 `Makefile.toml`
  2. 定义 `build` 任务
  3. 定义 `test` 任务
  4. 定义 `clippy` 任务
  5. 定义 `fmt` / `fmt-check` 任务
  6. 定义 `clean` 任务
- **Commit**: `chore: enable one-command build/test/lint workflow`
- **验收标准**:
  - [ ] `cargo make build` 成功
  - [ ] `cargo make test` 成功
  - [ ] `cargo make clippy` 成功
  - [ ] `cargo make fmt-check` 成功
- **代码审查**: 任务定义清晰

### CP1-T20: 创建 QEMU 运行任务
- **目标**: 在 Makefile.toml 中添加 QEMU 启动相关任务
- **开发任务**:
  1. 定义 `run` 任务（编译 + QEMU 启动）
  2. 定义 `run-headless` 任务（无 GUI）
  3. 定义 `run-trace` 任务（trace 捕获到文件）
  4. 定义 `debug` 任务（等待 GDB 连接）
- **Commit**: `chore: enable one-command QEMU boot and debug`
- **验收标准**:
  - [ ] 任务定义语法正确
  - [ ] QEMU 命令参数合理
- **代码审查**: 参数无硬编码路径问题

### CP1-T21: 配置 bootloader 依赖
- **目标**: 在 kernel crate 中正确配置 bootloader crate 依赖
- **开发任务**:
  1. 添加 `bootloader` 依赖到 kernel crate
  2. 配置 bootloader 的构建选项
  3. 确保 bootloader 能正确构建磁盘镜像
- **Commit**: `chore(kernel): enable bootable disk image generation`
- **验收标准**:
  - [ ] bootloader 依赖正确添加
  - [ ] 编译时能生成引导镜像
- **代码审查**: 版本兼容性检查

### CP1-T22: 创建 CI 配置文件
- **目标**: 创建 GitHub Actions CI 配置
- **开发任务**:
  1. 创建 `.github/workflows/ci.yml`
  2. 配置 build, test, clippy, fmt 步骤
  3. 配置 nightly toolchain 安装
- **Commit**: `chore: automate build/test/lint checks on every push`
- **验收标准**:
  - [ ] YAML 语法正确
  - [ ] 步骤覆盖编译、测试、lint、格式检查
- **代码审查**: CI 配置合理

### CP1-T23: 创建 .gitignore
- **目标**: 排除构建产物和 IDE 文件
- **开发任务**:
  1. 添加 `/target/`
  2. 添加 IDE 相关文件
  3. 添加 QEMU 临时文件
- **Commit**: `chore: exclude build artifacts from version control`
- **验收标准**:
  - [ ] target/ 被忽略
  - [ ] 编辑器配置被忽略
- **代码审查**: 无遗漏

### CP1-T24: 创建文档归档目录结构
- **目标**: 建立 docs/ 目录和模板文件
- **开发任务**:
  1. 创建 `docs/adr/` 目录和 `000-template.md`
  2. 创建 `docs/progress/` 目录
  3. 创建 `docs/risks/risk-register.md`
  4. 创建 `docs/decisions/pending-decisions.md`
  5. 创建 `CHANGELOG.md`
- **Commit**: `docs: establish documentation tracking infrastructure`
- **验收标准**:
  - [ ] 所有目录和模板文件存在
  - [ ] 模板格式与 plan.md 定义一致
- **代码审查**: 模板清晰可用

### CP1-T25: 创建第一个 ADR — 选择 Rust nightly
- **目标**: 记录使用 Rust nightly 的架构决策
- **开发任务**:
  1. 创建 `docs/adr/001-use-rust-nightly.md`
  2. 记录背景、决策、备选方案、影响
- **Commit**: `docs(adr): record why Rust nightly is required`
- **验收标准**:
  - [ ] ADR 格式正确
  - [ ] 内容完整（背景、决策、备选、影响）
- **代码审查**: 技术准确

### CP1-T26: 创建 ADR — 选择 bootloader crate
- **目标**: 记录引导方案选择
- **Commit**: `docs(adr): record bootloader strategy trade-offs`
- **验收标准**:
  - [ ] 对比了自写引导、bootloader crate、uefi-rs 三种方案

### CP1-T27: 创建 ADR — Workspace 多 crate 架构
- **目标**: 记录模块化架构决策
- **Commit**: `docs(adr): record why multi-crate beats single-crate`
- **验收标准**:
  - [ ] 说明了为什么选择多 crate 而非单 crate

### CP1-T28: 更新 README.md
- **目标**: 添加项目描述、构建说明、架构概览
- **开发任务**:
  1. 添加项目简介
  2. 添加先决条件（Rust nightly, QEMU）
  3. 添加快速开始（build, run, test）
  4. 添加项目结构概览
  5. 添加贡献指南链接
- **Commit**: `docs: enable new developers to build and run in 3 commands`
- **验收标准**:
  - [ ] 新开发者能根据 README 成功构建项目
  - [ ] 包含架构概览图
- **代码审查**: 清晰、完整

### CP1-T29: 在 common crate 定义 ColorCode 类型
- **目标**: VGA 文本模式的颜色代码类型
- **开发任务**:
  1. 定义 `Color` 枚举（16 色）
  2. 定义 `ColorCode` 结构体（前景 + 背景）
  3. 实现 `ColorCode::new(fg, bg)`
- **Commit**: `feat(common): enable type-safe VGA color composition`
- **验收标准**:
  - [ ] 16 种颜色定义
  - [ ] ColorCode 正确组合前景/背景色
  - [ ] 单元测试通过
- **代码审查**: 类型正确使用 `#[repr(u8)]`

### CP1-T30: 在 common crate 定义 ArrayString 类型
- **目标**: 固定大小的栈上字符串（no_std 环境不能用 String）
- **开发任务**:
  1. 实现 `ArrayString<const N: usize>` 结构体
  2. 实现 `new()`, `push()`, `as_str()`, `len()`, `is_empty()`
  3. 实现 `Display`, `Debug` trait
  4. 实现 `From<&str>` trait（截断超长输入）
- **Commit**: `feat(common): enable heap-free string storage in no_std`
- **验收标准**:
  - [ ] 编译通过（no_std）
  - [ ] 基本操作测试通过
  - [ ] 超长输入不 panic（截断处理）
- **代码审查**: 泛型 const 参数使用正确

### CP1-T31: 在 common crate 定义 SpanId / TraceId 类型
- **目标**: Trace 系统的基础 ID 类型
- **开发任务**:
  1. 定义 `TraceId(pub u64)`
  2. 定义 `SpanId(pub u64)`
  3. 实现 `Display`（十六进制格式）
  4. 实现 `PartialEq`, `Eq`, `Hash`, `Clone`, `Copy`
- **Commit**: `feat(common): ensure trace identifiers are type-safe and display-friendly`
- **验收标准**:
  - [ ] 类型定义正确
  - [ ] Display 输出十六进制格式
  - [ ] 单元测试通过
- **代码审查**: newtype 模式正确

### CP1-T32: 在 common crate 定义进程相关类型
- **目标**: 进程管理的基础类型
- **开发任务**:
  1. 定义 `Pid(pub u32)`
  2. 定义 `ProcessState` 枚举（Created, Ready, Running, Blocked, Terminated）
  3. 定义 `Priority(pub u8)` + 常量 (HIGH=0, MEDIUM=1, LOW=2, IDLE=3)
  4. 定义 `ProcessInfo` 结构体（摘要信息）
- **Commit**: `feat(common): define process lifecycle vocabulary types`
- **验收标准**:
  - [ ] 类型编译通过
  - [ ] ProcessState 覆盖所有状态
- **代码审查**: 类型设计简洁

### CP1-T33: 在 common crate 定义文件系统类型
- **目标**: 文件系统基础类型
- **开发任务**:
  1. 定义 `FileDescriptor(pub i32)`
  2. 定义 `InodeId(pub u64)`
  3. 定义 `OpenFlags` (bitflags)
  4. 定义 `SeekWhence` 枚举
  5. 定义 `DirEntry` 结构体
  6. 定义 `FileStat` 结构体
  7. 定义 `InodeType` 枚举（File, Directory, Device, Special）
- **Commit**: `feat(common): define filesystem vocabulary types`
- **验收标准**:
  - [ ] 类型编译通过
  - [ ] OpenFlags 使用 bitflags 宏
- **代码审查**: bitflags 使用正确

### CP1-T34: 在 common crate 定义 IPC 类型
- **目标**: 进程间通信基础类型
- **开发任务**:
  1. 定义 `QueueId(pub u32)`
  2. 定义 `ShmId(pub u32)`
  3. 定义 `Message` 结构体
- **Commit**: `feat(common): define IPC message vocabulary types`
- **验收标准**:
  - [ ] 类型编译通过
  - [ ] Message 中包含 TraceContext 字段
- **代码审查**: 消息大小合理

### CP1-T35: 在 common crate 定义错误类型
- **目标**: 所有子系统的错误枚举
- **开发任务**:
  1. 定义 `MemoryError` 枚举
  2. 定义 `ProcessError` 枚举
  3. 定义 `FsError` 枚举
  4. 定义 `IpcError` 枚举
  5. 定义 `DriverError` 枚举
  6. 定义 `TraceError` 枚举
  7. 定义 `KernelError` 统一枚举（From 转换）
- **Commit**: `feat(common): enable typed error handling across subsystems`
- **验收标准**:
  - [ ] 所有错误类型实现 `Debug`
  - [ ] `KernelError` 实现 `From<XxxError>` 转换
  - [ ] 错误变体覆盖 spec.md 定义的所有情况
- **代码审查**: 错误粒度适当

### CP1-T36: 在 common crate 定义 Trait — FrameAllocator
- **目标**: 定义物理帧分配器接口
- **Commit**: `feat(common): establish frame allocation contract`
- **验收标准**:
  - [ ] Trait 方法签名与 spec.md 一致
  - [ ] rustdoc 文档完整

### CP1-T37: 在 common crate 定义 Trait — VirtualMemoryManager
- **目标**: 定义虚拟内存管理接口
- **Commit**: `feat(common): establish virtual memory management contract`
- **验收标准**:
  - [ ] Trait 方法签名与 spec.md 一致

### CP1-T38: 在 common crate 定义 Trait — Tracer
- **目标**: 定义 Trace 引擎接口
- **Commit**: `feat(common): establish trace engine contract`
- **验收标准**:
  - [ ] 包含 begin_span, end_span, add_event, current_context 等方法
  - [ ] SpanGuard 类型定义

### CP1-T39: 在 common crate 定义 Trait — ProcessManager
- **目标**: 定义进程管理接口
- **Commit**: `feat(common): establish process management contract`
- **验收标准**:
  - [ ] 方法签名与 spec.md 一致

### CP1-T40: 在 common crate 定义 Trait — Scheduler
- **目标**: 定义调度器接口
- **Commit**: `feat(common): establish task scheduling contract`
- **验收标准**:
  - [ ] ScheduleDecision 枚举定义
  - [ ] SchedulerStats 结构体定义

### CP1-T41: 在 common crate 定义 Trait — FileSystem + FileSystemDriver
- **目标**: 定义文件系统层接口
- **Commit**: `feat(common): establish VFS and driver-level filesystem contracts`
- **验收标准**:
  - [ ] 双层 trait 设计（VFS + Driver）

### CP1-T42: 在 common crate 定义 Trait — IpcManager
- **目标**: 定义 IPC 接口
- **Commit**: `feat(common): establish IPC communication contract`
- **验收标准**:
  - [ ] 消息队列和共享内存方法

### CP1-T43: 在 common crate 定义 Trait — DeviceDriver
- **目标**: 定义设备驱动接口
- **Commit**: `feat(common): establish device driver contract`
- **验收标准**:
  - [ ] DeviceType 枚举
  - [ ] init/read/write/ioctl 方法

### CP1-T44: 在 common crate 定义 Trait — HAL traits
- **目标**: 定义 HAL 层接口（HalSerial, HalDisplay, HalInterruptController）
- **Commit**: `feat(common): establish hardware abstraction contracts`
- **验收标准**:
  - [ ] 三个 trait 定义完整

### CP1-T45: 定义 KernelServices 注册表结构
- **目标**: 定义全局内核服务注册表
- **开发任务**:
  1. 在 common crate 定义 `KernelServices` 结构体
  2. 定义 `kernel()` 全局访问函数
  3. 使用 `Once` 或类似机制保证单次初始化
- **Commit**: `feat(common): enable decoupled subsystem access via registry`
- **验收标准**:
  - [ ] KernelServices 包含所有子系统 trait 引用
  - [ ] `kernel()` 函数返回 `&'static KernelServices`
- **代码审查**: 线程安全性正确

### CP1-T46: 验证完整 workspace 编译
- **目标**: 确保所有 crate 协同编译通过
- **开发任务**:
  1. 修复所有编译错误
  2. 修复所有 clippy 警告
  3. 运行 fmt
- **Commit**: `fix: ensure all crates compile together without errors`
- **验收标准**:
  - [ ] `cargo build --workspace --target x86_64-unknown-none` 通过
  - [ ] `cargo clippy --workspace` 零警告
  - [ ] `cargo fmt --all --check` 通过
- **代码审查**: 无临时 hack

### CP1-T47: 创建启动脚本
- **目标**: 创建便捷脚本
- **开发任务**:
  1. 创建 `scripts/run.sh`
  2. 创建 `scripts/debug.sh`
  3. 创建 `scripts/capture-trace.sh`
  4. 设置可执行权限
- **Commit**: `chore: simplify common dev operations to single commands`
- **验收标准**:
  - [ ] 脚本语法正确
  - [ ] 有执行权限
- **代码审查**: 脚本有错误处理

### CP1-T48: CP1 完成报告
- **目标**: 生成检查点完成报告，更新 CHANGELOG
- **开发任务**:
  1. 创建 `docs/progress/cp01-report.md`
  2. 更新 `CHANGELOG.md`
  3. 更新 AGENTS.md
- **Commit**: `docs: record CP1 outcomes and quality metrics`
- **验收标准**:
  - [ ] 报告包含所有验收标准的达成情况
  - [ ] CHANGELOG 记录了所有变更

---

# CP2: HAL 硬件抽象层 + 引导

> **目标**: 实现完整的 HAL 层，内核能在 QEMU 中引导并输出 "Hello, MiniOS!"
> **预计任务数**: 52
> **前置条件**: CP1 全部完成

## CP2 验收标准

- [ ] **AC-2.1**: `cargo make run` 在 QEMU 中启动，VGA 显示 "Hello, MiniOS!"
- [ ] **AC-2.2**: 串口输出引导日志信息
- [ ] **AC-2.3**: GDT 和 TSS 正确配置
- [ ] **AC-2.4**: VGA 文本模式支持彩色输出和自动滚屏
- [ ] **AC-2.5**: 串口支持读写
- [ ] **AC-2.6**: `println!` 和 `serial_println!` 宏可用
- [ ] **AC-2.7**: panic handler 在 VGA 和串口同时输出
- [ ] **AC-2.8**: HAL 所有函数有 rustdoc 文档
- [ ] **AC-2.9**: HAL 模块单元测试通过
- [ ] **AC-2.10**: 代码通过 clippy + fmt 检查

## CP2 预检查

```
开始 CP2 前检查:
- [ ] CP1 所有验收标准满足
- [ ] cargo build 通过
- [ ] cargo test 通过
- [ ] 是否需要重构 common crate 的类型定义？
```

## CP2 任务列表

### CP2-T01: 实现 Port I/O 封装
- **目标**: 安全封装 x86 端口读写操作
- **开发任务**:
  1. 在 `crates/hal/src/port.rs` 实现 `Port<T>` 结构体
  2. 实现 `PortReadOnly<T>` 和 `PortWriteOnly<T>`
  3. 泛型 T 支持 u8, u16, u32
  4. 使用 `x86_64` crate 的端口 I/O 或直接内联汇编
- **Commit**: `feat(hal): enable safe x86 port read/write`
- **验收标准**:
  - [ ] 编译通过
  - [ ] unsafe 代码有 SAFETY 注释
  - [ ] rustdoc 完整
- **代码审查**: unsafe 范围最小化

### CP2-T02: 实现串口驱动 — 初始化
- **目标**: 初始化 UART 16550 串口控制器
- **开发任务**:
  1. 在 `crates/hal/src/serial.rs` 定义 `SerialPort` 结构体
  2. 实现 `init()` 方法：配置波特率、数据位、停止位
  3. COM1 端口地址: 0x3F8
- **Commit**: `feat(hal): enable UART 16550 serial communication`
- **验收标准**:
  - [ ] 初始化代码正确设置串口寄存器
  - [ ] 使用 Port I/O 封装（CP2-T01 的成果）
- **代码审查**: 寄存器地址正确

### CP2-T03: 实现串口驱动 — 写入
- **目标**: 向串口发送字节/字符串
- **开发任务**:
  1. 实现 `write_byte(byte: u8)` — 等待发送缓冲区空再写
  2. 实现 `write_string(s: &str)`
  3. 实现 `fmt::Write` trait
- **Commit**: `feat(hal): enable formatted text output via serial`
- **验收标准**:
  - [ ] 实现 `fmt::Write` 以支持 `write!` 宏
  - [ ] 等待发送就绪（检查 Line Status Register）
- **代码审查**: 繁忙等待有边界保护

### CP2-T04: 实现串口驱动 — 读取
- **目标**: 从串口读取字节
- **开发任务**:
  1. 实现 `read_byte() -> Option<u8>` — 检查是否有数据可读
- **Commit**: `feat(hal): implement serial port read`
- **验收标准**:
  - [ ] 非阻塞读取（无数据返回 None）
- **代码审查**: 寄存器位掩码正确

### CP2-T05: 创建串口全局实例和宏
- **目标**: 提供 `serial_print!` 和 `serial_println!` 宏
- **开发任务**:
  1. 创建 `static SERIAL1: Mutex<SerialPort>`
  2. 定义 `serial_print!` 宏
  3. 定义 `serial_println!` 宏
  4. 定义 `_serial_print` 辅助函数
- **Commit**: `feat(hal): add serial_print! and serial_println! macros`
- **验收标准**:
  - [ ] 宏在任何模块中可用
  - [ ] 线程安全（使用 spin::Mutex）
- **代码审查**: 宏导出正确

### CP2-T06: 实现 VGA 文本缓冲区 — 基础结构
- **目标**: 定义 VGA 文本模式的内存映射结构
- **开发任务**:
  1. 在 `crates/hal/src/vga.rs` 定义 `ScreenChar` 结构体 (char + color)
  2. 定义 `Buffer` 结构体 (80x25 ScreenChar 数组)
  3. 定义 `Writer` 结构体 (持有 Buffer 引用 + 当前位置 + 颜色)
  4. VGA 缓冲区地址: 0xB8000
- **Commit**: `feat(hal): define VGA text buffer data structures`
- **验收标准**:
  - [ ] ScreenChar 使用 `#[repr(C)]`
  - [ ] Buffer 使用 `Volatile` 包装防止优化
- **代码审查**: 内存布局正确

### CP2-T07: 实现 VGA — 写入字符
- **目标**: 向 VGA 缓冲区写入单个字符
- **开发任务**:
  1. 实现 `Writer::write_byte(byte: u8)`
  2. 处理换行符 `\n`
  3. 处理不可打印字符（替换为 `0xFE`）
  4. 自动换行（到达行末）
- **Commit**: `feat(hal): implement VGA character writing`
- **验收标准**:
  - [ ] 换行正确处理
  - [ ] 行末自动换行
  - [ ] 不可打印字符安全处理
- **代码审查**: 边界检查完整

### CP2-T08: 实现 VGA — 滚屏
- **目标**: 当写满最后一行时向上滚动屏幕
- **开发任务**:
  1. 实现 `Writer::scroll_up()`
  2. 将每行的内容复制到上一行
  3. 清空最后一行
- **Commit**: `feat(hal): implement VGA scroll up`
- **验收标准**:
  - [ ] 滚屏后第一行被丢弃
  - [ ] 最后一行被清空
  - [ ] 使用 volatile 写入
- **代码审查**: 复制方向正确（避免覆盖）

### CP2-T09: 实现 VGA — 颜色设置和清屏
- **目标**: 支持颜色切换和屏幕清除
- **开发任务**:
  1. 实现 `Writer::set_color(fg: Color, bg: Color)`
  2. 实现 `Writer::clear()`
- **Commit**: `feat(hal): implement VGA color and clear screen`
- **验收标准**:
  - [ ] 颜色设置后影响后续写入
  - [ ] 清屏用空格填充整个缓冲区
- **代码审查**: 颜色位组合正确

### CP2-T10: 实现 VGA — fmt::Write trait 和宏
- **目标**: 提供 `print!` 和 `println!` 宏
- **开发任务**:
  1. 为 `Writer` 实现 `fmt::Write` trait
  2. 创建全局 `static WRITER: Mutex<Writer>`
  3. 定义 `print!` 和 `println!` 宏
  4. 定义 `_print` 辅助函数
- **Commit**: `feat(hal): add print! and println! macros for VGA`
- **验收标准**:
  - [ ] `println!("Hello, {}!", "MiniOS")` 格式化正确
  - [ ] 线程安全
- **代码审查**: 宏导出路径正确

### CP2-T11: 实现 VGA HalDisplay trait
- **目标**: 为 VGA Writer 实现 common 中定义的 HalDisplay trait
- **开发任务**:
  1. 实现 `HalDisplay for VgaDisplay`
  2. 桥接到内部 Writer 实现
- **Commit**: `feat(hal): implement HalDisplay trait for VGA`
- **验收标准**:
  - [ ] 所有 trait 方法已实现
  - [ ] 可通过 trait 对象使用
- **代码审查**: trait 实现与定义一致

### CP2-T12: 实现串口 HalSerial trait
- **目标**: 为 SerialPort 实现 HalSerial trait
- **Commit**: `feat(hal): implement HalSerial trait for UART`
- **验收标准**:
  - [ ] 所有方法实现
  - [ ] 可通过 trait 对象使用

### CP2-T13: 实现 GDT — 定义段描述符
- **目标**: 创建全局描述符表
- **开发任务**:
  1. 在 `crates/hal/src/gdt.rs` 使用 `x86_64` crate 的 GDT API
  2. 定义内核代码段
  3. 定义内核数据段
  4. 定义 TSS 段
- **Commit**: `feat(hal): define GDT with kernel segments`
- **验收标准**:
  - [ ] GDT 包含必需段（代码、数据、TSS）
  - [ ] 使用 x86_64 crate 的类型安全 API
- **代码审查**: 段权限正确

### CP2-T14: 实现 TSS — 任务状态段
- **目标**: 配置 TSS 用于中断栈切换
- **开发任务**:
  1. 定义 TSS 结构体
  2. 设置 IST (Interrupt Stack Table) 条目
  3. 为 Double Fault 分配独立的中断栈
- **Commit**: `feat(hal): configure TSS with interrupt stack`
- **验收标准**:
  - [ ] IST[0] 设置了 Double Fault 栈
  - [ ] 栈大小合理（至少 4 KiB）
- **代码审查**: 栈地址对齐正确

### CP2-T15: 实现 GDT 加载
- **目标**: 将 GDT 加载到 CPU 并设置段寄存器
- **开发任务**:
  1. 实现 `gdt::init()` 函数
  2. 加载 GDT (`lgdt`)
  3. 加载 TSS (`ltr`)
  4. 重载代码段寄存器 (`cs`)
  5. 设置数据段寄存器
- **Commit**: `feat(hal): load GDT and set segment registers`
- **验收标准**:
  - [ ] GDT 加载后 CPU 正常运行
  - [ ] 段选择器正确
- **代码审查**: unsafe 操作有 SAFETY 注释

### CP2-T16: 实现 PIC 8259 初始化
- **目标**: 初始化级联 PIC 中断控制器
- **开发任务**:
  1. 在 `crates/hal/src/pic.rs` 定义 `ChainedPics` 结构体
  2. 实现 `init()` — ICW1-ICW4 初始化序列
  3. 设置中断偏移量（主 PIC: 32-39, 从 PIC: 40-47）
  4. 默认屏蔽所有中断
- **Commit**: `feat(hal): implement PIC 8259 initialization`
- **验收标准**:
  - [ ] PIC 正确初始化
  - [ ] 中断偏移避开 CPU 异常（>=32）
- **代码审查**: 初始化序列顺序正确

### CP2-T17: 实现 PIC — EOI 和中断屏蔽
- **目标**: PIC 中断结束信号和 IRQ 控制
- **开发任务**:
  1. 实现 `end_of_interrupt(irq: u8)`
  2. 实现 `enable_irq(irq: u8)` — 清除 IMR 中对应 bit
  3. 实现 `disable_irq(irq: u8)` — 设置 IMR 中对应 bit
- **Commit**: `feat(hal): implement PIC EOI and IRQ masking`
- **验收标准**:
  - [ ] EOI 正确发送到主/从 PIC
  - [ ] IRQ 屏蔽/启用正确操作位
- **代码审查**: 主从 PIC 判断逻辑正确

### CP2-T18: 实现 PIC HalInterruptController trait
- **目标**: 为 PIC 实现 HAL trait
- **Commit**: `feat(hal): implement HalInterruptController for PIC`
- **验收标准**:
  - [ ] trait 方法全部实现

### CP2-T19: 实现 CPU 控制函数
- **目标**: 封装常用 CPU 操作
- **开发任务**:
  1. 在 `crates/hal/src/cpu.rs` 实现 `enable_interrupts()` — `sti`
  2. 实现 `disable_interrupts()` — `cli`
  3. 实现 `hlt()` — 休眠等待中断
  4. 实现 `halt_loop()` — `cli; hlt` 循环
  5. 实现 `read_tsc() -> u64` — 读取时间戳计数器
  6. 实现 `without_interrupts(f)` — 临界区执行
- **Commit**: `feat(hal): implement CPU control functions`
- **验收标准**:
  - [ ] 所有函数有 unsafe SAFETY 注释
  - [ ] read_tsc 正确使用 rdtsc 指令
  - [ ] without_interrupts 保证中断恢复
- **代码审查**: 中断状态正确保存/恢复

### CP2-T20: 创建 HAL 统一初始化入口
- **目标**: `hal::init()` 函数统一初始化所有 HAL 组件
- **开发任务**:
  1. 实现 `hal::init()` 函数
  2. 调用顺序: GDT → 串口 → VGA → PIC
  3. 每步输出日志到串口
- **Commit**: `feat(hal): add unified hal::init() entry point`
- **验收标准**:
  - [ ] 初始化顺序正确
  - [ ] 初始化失败会 panic 并输出错误位置
- **代码审查**: 初始化顺序依赖关系正确

### CP2-T21: 实现 kernel 入口点 — _start
- **目标**: 实现内核入口函数
- **开发任务**:
  1. 在 `crates/kernel/src/main.rs` 使用 bootloader 的入口宏
  2. 接收 `BootInfo` 参数
  3. 调用 `hal::init()`
  4. 输出 "Hello, MiniOS!" 到 VGA
  5. 输出引导日志到串口
  6. 进入 halt loop
- **Commit**: `feat(kernel): implement kernel entry point with boot message`
- **验收标准**:
  - [ ] 内核入口函数签名正确
  - [ ] VGA 显示 "Hello, MiniOS!"
  - [ ] 串口输出引导日志
- **代码审查**: bootloader API 使用正确

### CP2-T22: 实现 panic handler
- **目标**: 内核 panic 时的处理逻辑
- **开发任务**:
  1. 在 `crates/kernel/src/panic.rs` 实现 `#[panic_handler]`
  2. 输出 panic 信息到 VGA（红色背景）
  3. 输出 panic 信息到串口
  4. 进入 halt loop
- **Commit**: `feat(kernel): implement panic handler with VGA and serial output`
- **验收标准**:
  - [ ] panic 信息包含文件名和行号
  - [ ] VGA 显示红色背景错误信息
  - [ ] 串口也输出 panic 信息
- **代码审查**: panic handler 不依赖可能已损坏的状态

### CP2-T23: 配置 bootloader 构建并生成引导镜像
- **目标**: 使编译产物可在 QEMU 中引导
- **开发任务**:
  1. 配置 bootloader 的 `boot` 功能
  2. 设置磁盘镜像生成
  3. 更新 Makefile.toml 的 run 任务指向正确的镜像文件
- **Commit**: `chore(kernel): configure bootloader image generation`
- **验收标准**:
  - [ ] `cargo build` 后生成可引导镜像
  - [ ] 镜像格式正确（BIOS 或 UEFI）
- **代码审查**: 路径配置正确

### CP2-T24: 首次 QEMU 引导测试
- **目标**: 在 QEMU 中成功引导并显示消息
- **开发任务**:
  1. 安装 QEMU (如未安装)
  2. 运行 `cargo make run`
  3. 验证 VGA 输出
  4. 验证串口输出
  5. 修复所有引导问题
- **Commit**: `fix(kernel): resolve boot issues for QEMU` (如有修复)
- **验收标准**:
  - [ ] QEMU 窗口显示 "Hello, MiniOS!"
  - [ ] 串口输出引导日志
  - [ ] 无 Triple Fault
- **代码审查**: N/A（集成测试）

### CP2-T25: 实现 QEMU 退出机制
- **目标**: 测试时能优雅退出 QEMU
- **开发任务**:
  1. 实现 `exit_qemu(exit_code: QemuExitCode)`
  2. 使用 QEMU 的 `isa-debug-exit` 设备
  3. 更新 QEMU 命令行参数添加 `-device isa-debug-exit,iobase=0xf4,iosize=0x04`
- **Commit**: `feat(hal): implement QEMU exit mechanism for testing`
- **验收标准**:
  - [ ] `exit_qemu(Success)` 正确退出 QEMU
  - [ ] 退出码可区分成功/失败
- **代码审查**: iobase 地址不冲突

### CP2-T26: 实现自定义测试框架
- **目标**: 在 no_std 环境中运行测试
- **开发任务**:
  1. 添加 `#![feature(custom_test_frameworks)]`
  2. 定义 `test_runner` 函数
  3. 定义 `Testable` trait
  4. 配置测试时通过串口输出结果
  5. 测试完成后退出 QEMU
- **Commit**: `feat(kernel): implement custom test framework for no_std`
- **验收标准**:
  - [ ] `cargo test` 能在 QEMU 中运行并自动退出
  - [ ] 测试结果通过串口输出
  - [ ] 退出码反映测试成功/失败
- **代码审查**: test runner 逻辑正确

### CP2-T27: 添加 VGA 基础测试
- **目标**: VGA 输出的单元测试
- **开发任务**:
  1. 测试 `println!` 不 panic
  2. 测试长字符串输出（触发滚屏）
  3. 测试格式化输出
- **Commit**: `test(hal): add VGA output tests`
- **验收标准**:
  - [ ] 3+ 个测试用例
  - [ ] 所有测试通过
- **代码审查**: 测试有意义

### CP2-T28: 添加串口基础测试
- **目标**: 串口输出的单元测试
- **Commit**: `test(hal): add serial port tests`
- **验收标准**:
  - [ ] 基本写入测试通过

### CP2-T29: 添加 GDT/TSS 测试
- **目标**: 验证 GDT 加载不 crash
- **Commit**: `test(hal): add GDT initialization test`
- **验收标准**:
  - [ ] GDT 初始化后系统不 triple fault

### CP2-T30 ~ CP2-T40: 代码整理和优化

### CP2-T30: 重构 HAL 模块导出
- **目标**: 清理 HAL lib.rs 的公共接口
- **Commit**: `refactor(hal): organize public module exports`
- **验收标准**:
  - [ ] lib.rs 清晰列出所有子模块
  - [ ] 公共接口有 `pub use` 重导出

### CP2-T31: 为 HAL 所有公开接口添加 rustdoc
- **目标**: 文档完整性
- **Commit**: `docs(hal): add rustdoc for all public APIs`
- **验收标准**:
  - [ ] 每个 pub fn/struct/enum/trait 有 `///` 文档
  - [ ] `cargo doc` 生成无警告

### CP2-T32: 实现 HAL 错误处理
- **目标**: 统一 HAL 层的错误处理方式
- **Commit**: `feat(hal): add HAL error types and handling`
- **验收标准**:
  - [ ] 硬件操作失败有明确错误类型

### CP2-T33: 优化 VGA Writer 性能
- **目标**: 减少不必要的 volatile 操作
- **Commit**: `refactor(hal): optimize VGA writer performance`
- **验收标准**:
  - [ ] 滚屏不逐字节操作

### CP2-T34: 添加 VGA 光标控制
- **目标**: 控制硬件光标位置
- **开发任务**:
  1. 实现 `enable_cursor()`
  2. 实现 `set_cursor_position(row, col)`
  3. 实现 `disable_cursor()`
- **Commit**: `feat(hal): add VGA hardware cursor control`
- **验收标准**:
  - [ ] 光标跟随最新写入位置
- **代码审查**: 光标寄存器操作正确

### CP2-T35: 添加颜色常量和预设
- **目标**: 常用颜色组合
- **Commit**: `feat(hal): add color presets for VGA`
- **验收标准**:
  - [ ] 预设包含：正常、错误、警告、高亮

### CP2-T36: 实现 VGA 的行/列操作
- **目标**: 支持定位输出
- **开发任务**:
  1. `write_at(row, col, char, color)`
  2. `clear_row(row)`
  3. `get_position() -> (row, col)`
- **Commit**: `feat(hal): add positioned VGA write operations`
- **验收标准**:
  - [ ] 定位写入正确
  - [ ] 边界检查完整

### CP2-T37: 实现串口的日志级别支持
- **目标**: 支持不同级别的日志输出（为后续 trace 做铺垫）
- **开发任务**:
  1. 定义 `LogLevel` 枚举 (Trace, Debug, Info, Warn, Error)
  2. 实现 `log!(level, ...)` 宏
  3. 支持编译时过滤级别
- **Commit**: `feat(hal): add log level support for serial output`
- **验收标准**:
  - [ ] 5 个日志级别
  - [ ] 输出包含级别标签和时间戳

### CP2-T38: 在引导流程中添加详细日志
- **目标**: 引导每一步的详细日志输出
- **Commit**: `feat(kernel): add detailed boot logging`
- **验收标准**:
  - [ ] 每个初始化步骤有 `[OK]` / `[FAIL]` 标记

### CP2-T39: 实现引导信息显示
- **目标**: 在 VGA 上显示引导信息摘要
- **开发任务**:
  1. 显示 MiniOS 版本和构建时间
  2. 显示内存信息（从 BootInfo 读取）
  3. 显示 CPU 信息
- **Commit**: `feat(kernel): display boot info summary on VGA`
- **验收标准**:
  - [ ] VGA 显示格式化的引导信息
  - [ ] 包含可用内存总量

### CP2-T40: 引导输出美化
- **目标**: 美化 VGA 引导输出（彩色 banner）
- **Commit**: `feat(kernel): add colorful boot banner`
- **验收标准**:
  - [ ] 显示 MiniOS ASCII Art logo
  - [ ] 使用彩色输出

### CP2-T41 ~ CP2-T52: 健壮性和集成

### CP2-T41: 实现 without_interrupts 安全包装
- **目标**: 确保中断安全的临界区辅助函数
- **Commit**: `feat(hal): implement interrupt-safe critical section wrapper`
- **验收标准**:
  - [ ] 中断状态在退出时正确恢复

### CP2-T42: 实现串口的 Write trait（for core::fmt）
- **目标**: 使串口可用于格式化输出
- **Commit**: `feat(hal): implement core::fmt::Write for serial`
- **验收标准**: 格式化输出正确

### CP2-T43: 添加 boot_info 解析功能
- **目标**: 解析 bootloader 提供的启动信息
- **开发任务**:
  1. 解析内存映射表
  2. 识别可用/已用/保留区域
  3. 计算总可用内存
- **Commit**: `feat(kernel): parse boot info memory map`
- **验收标准**:
  - [ ] 正确解析所有内存区域类型
  - [ ] 日志输出内存映射

### CP2-T44: 修复编译警告
- **目标**: 清理所有 clippy 警告
- **Commit**: `fix: resolve all clippy warnings`
- **验收标准**:
  - [ ] `cargo clippy` 零警告

### CP2-T45: 代码格式化
- **目标**: 确保代码格式统一
- **Commit**: `chore: apply rustfmt to all crates`
- **验收标准**:
  - [ ] `cargo fmt --check` 通过

### CP2-T46: 确保所有 crate 仍可编译
- **目标**: CP2 变更没有破坏其他 crate
- **验收标准**:
  - [ ] `cargo build --workspace` 通过

### CP2-T47: 创建集成测试
- **目标**: 整体引导测试
- **开发任务**:
  1. 创建 `tests/boot_test.rs`
  2. 测试：内核引导不 triple fault
  3. 测试：VGA 和串口输出工作
- **Commit**: `test(kernel): add boot integration test`
- **验收标准**:
  - [ ] 集成测试通过

### CP2-T48: 创建 ADR — HAL 层设计选择
- **Commit**: `docs(adr): ADR-004 HAL layer design`
- **验收标准**: ADR 格式正确

### CP2-T49: 更新 README — 运行说明
- **Commit**: `docs: update README with QEMU run instructions`
- **验收标准**: 包含截图或输出示例

### CP2-T50: 更新 AGENTS.md
- **Commit**: `docs: update AGENTS.md with HAL development notes`
- **验收标准**: 包含构建和运行命令

### CP2-T51: 更新 CHANGELOG
- **Commit**: `docs: update CHANGELOG for CP2`
- **验收标准**: 记录所有新增功能

### CP2-T52: CP2 完成报告
- **Commit**: `docs: add CP2 completion report`
- **验收标准**: 报告模板完整填写

---

# CP3: Trace 引擎

> **目标**: 实现完整的 Trace 引擎，包括 Ring Buffer、Span 管理、TraceContext 传播、trace 宏，并集成到引导流程
> **预计任务数**: 52
> **前置条件**: CP2 全部完成

## CP3 验收标准

- [ ] **AC-3.1**: `trace_span!` 宏在内核代码中可用，创建 span 并自动结束
- [ ] **AC-3.2**: `trace_event!` 宏可记录瞬时事件
- [ ] **AC-3.3**: Ring Buffer 正确存储 span 数据，支持覆盖最旧数据
- [ ] **AC-3.4**: TraceContext 在嵌套调用中正确传播 parent_span_id
- [ ] **AC-3.5**: Trace 数据可通过串口以 JSON 格式导出
- [ ] **AC-3.6**: 引导流程的每一步都有 trace span
- [ ] **AC-3.7**: Trace 引擎的单元测试通过（覆盖正常/边界/错误路径）
- [ ] **AC-3.8**: Trace 开销 < 500ns per span（首版目标，后续优化到 <100ns）
- [ ] **AC-3.9**: SpanFilter 支持按 module/name/trace_id 过滤
- [ ] **AC-3.10**: TraceStats 统计信息正确

## CP3 任务列表

### CP3-T01: 实现 Span 数据结构
- **目标**: 定义 trace span 的内存布局
- **开发任务**:
  1. 在 `crates/trace/src/span.rs` 定义 `Span` 结构体
  2. 实现 `#[repr(C)]` 确保固定布局
  3. 实现 `Span::new()`, `Span::default()`
  4. 实现 `SpanStatus` 枚举
  5. 确保单个 Span 大小固定（约 512 字节）
- **Commit**: `feat(trace): implement Span data structure`
- **验收标准**:
  - [ ] Span 大小固定且已知 (`core::mem::size_of`)
  - [ ] 所有字段有明确用途
  - [ ] 单元测试验证大小和布局
- **代码审查**: repr(C) 正确，字段对齐合理

### CP3-T02: 实现 SpanAttributes — 零分配属性存储
- **目标**: 固定大小的 key-value 属性存储
- **开发任务**:
  1. 定义 `AttributeValue` 枚举 (U64, I64, Str, Bool)
  2. 定义 `SpanAttributes` 结构体（最多 8 个条目）
  3. 实现 `set(key, value)` 和 `get(key)` 方法
  4. 实现 `From` trait 用于常见类型转换
- **Commit**: `feat(trace): implement SpanAttributes with fixed-size storage`
- **验收标准**:
  - [ ] 零堆分配
  - [ ] 超过 8 个属性时静默忽略（不 panic）
  - [ ] 单元测试覆盖边界条件
- **代码审查**: 无隐式分配

### CP3-T03: 实现 TraceContext
- **目标**: 跟踪当前调用链的上下文
- **开发任务**:
  1. 在 `crates/trace/src/context.rs` 定义 `TraceContext`
  2. 实现 per-CPU/per-task 的上下文存储（初期用全局变量）
  3. 实现 `push_span()` / `pop_span()` 栈操作
  4. 实现最大深度保护（64 层）
- **Commit**: `feat(trace): implement TraceContext with span stack`
- **验收标准**:
  - [ ] 嵌套 span 正确设置 parent_span_id
  - [ ] 深度超过 64 层时不 panic
  - [ ] 单元测试验证嵌套关系
- **代码审查**: 线程安全性考虑

### CP3-T04: 实现 SpanId 生成器
- **目标**: 全局唯一的 SpanId 分配
- **开发任务**:
  1. 使用 AtomicU64 递增计数器
  2. 实现 `next_span_id() -> SpanId`
- **Commit**: `feat(trace): implement atomic SpanId generator`
- **验收标准**:
  - [ ] ID 全局唯一
  - [ ] 线程安全（原子操作）
- **代码审查**: 内存序正确

### CP3-T05: 实现 TraceId 生成器
- **目标**: 全局唯一的 TraceId 分配
- **Commit**: `feat(trace): implement TraceId generator`
- **验收标准**:
  - [ ] 每个新的顶级 span 获得新 TraceId
  - [ ] 嵌套 span 继承父 TraceId

### CP3-T06: 实现 Ring Buffer — 基础结构
- **目标**: 固定大小的环形缓冲区
- **开发任务**:
  1. 在 `crates/trace/src/ringbuffer.rs` 定义 `TraceRingBuffer`
  2. 使用固定大小的 Span 数组
  3. 使用 AtomicUsize 作为写指针
  4. 实现 `write(span: Span)` — 无锁写入
- **Commit**: `feat(trace): implement Ring Buffer core structure`
- **验收标准**:
  - [ ] 写入使用原子操作
  - [ ] 缓冲区满时覆盖最旧数据
  - [ ] 单元测试验证覆盖行为
- **代码审查**: 无锁设计正确

### CP3-T07: 实现 Ring Buffer — 读取
- **目标**: 从 Ring Buffer 读取 span 数据
- **开发任务**:
  1. 实现 `read_all(out: &mut [Span]) -> usize`
  2. 实现 `read_recent(count: usize, out: &mut [Span]) -> usize`
  3. 处理读取时的写入竞争
- **Commit**: `feat(trace): implement Ring Buffer read operations`
- **验收标准**:
  - [ ] 读取不影响写入
  - [ ] 返回正确的 span 数量
- **代码审查**: 读写竞争处理正确

### CP3-T08: 实现 Ring Buffer — 统计
- **目标**: 缓冲区使用统计
- **开发任务**:
  1. 追踪 total_written（原子计数器）
  2. 实现 `stats() -> BufferStats`（容量、已用、已写总数、覆盖次数）
- **Commit**: `feat(trace): add Ring Buffer statistics`
- **验收标准**:
  - [ ] 统计数据准确
- **代码审查**: 原子操作正确

### CP3-T09: 实现 SpanFilter
- **目标**: trace 数据过滤器
- **开发任务**:
  1. 在 `crates/trace/src/filter.rs` 定义 `SpanFilter`
  2. 支持按 module 过滤
  3. 支持按 name 过滤
  4. 支持按 trace_id 过滤
  5. 支持按 pid 过滤
  6. 支持按状态过滤 (Ok/Error)
  7. 实现 `matches(span: &Span) -> bool`
- **Commit**: `feat(trace): implement SpanFilter with multiple criteria`
- **验收标准**:
  - [ ] 各过滤条件独立生效
  - [ ] 多条件组合（AND 逻辑）
  - [ ] 单元测试覆盖各过滤场景
- **代码审查**: 过滤逻辑清晰

### CP3-T10: 实现 Ring Buffer — 过滤读取
- **目标**: 支持带过滤器的读取
- **开发任务**:
  1. 实现 `read_filtered(filter: &SpanFilter, out: &mut [Span]) -> usize`
- **Commit**: `feat(trace): add filtered read to Ring Buffer`
- **验收标准**:
  - [ ] 过滤结果正确
  - [ ] 性能可接受

### CP3-T11: 实现 Tracer Engine — 核心逻辑
- **目标**: 实现 Tracer trait 的核心引擎
- **开发任务**:
  1. 在 `crates/trace/src/engine.rs` 定义 `TraceEngine` 结构体
  2. 持有 Ring Buffer 和 TraceContext
  3. 实现 `Tracer` trait 的所有方法
  4. 实现 `begin_span` — 创建 span、推入上下文、写入 buffer
  5. 实现 `end_span` — 设置 end_tsc、更新状态、弹出上下文
- **Commit**: `feat(trace): implement TraceEngine core logic`
- **验收标准**:
  - [ ] Tracer trait 完全实现
  - [ ] begin_span + end_span 正确配对
  - [ ] TraceContext 正确管理
- **代码审查**: span 生命周期正确

### CP3-T12: 实现 SpanGuard — RAII 自动结束
- **目标**: Drop 时自动结束 span
- **开发任务**:
  1. 定义 `SpanGuard` 结构体
  2. 实现 `Drop` trait — 调用 `tracer.end_span()`
  3. 实现 `set_status()`, `add_attribute()` 方法
- **Commit**: `feat(trace): implement SpanGuard RAII for automatic span end`
- **验收标准**:
  - [ ] 正常退出和 early return 都能结束 span
  - [ ] panic 时也能结束 span（Drop 保证）
- **代码审查**: 不可 Copy/Clone

### CP3-T13: 实现 trace_span! 宏
- **目标**: 便捷的 span 创建宏
- **开发任务**:
  1. 宏签名: `trace_span!(name, module = "xxx", key = value, ...)`
  2. 返回 SpanGuard
  3. 支持可选的 key-value 属性
- **Commit**: `feat(trace): implement trace_span! macro`
- **验收标准**:
  - [ ] 宏语法简洁
  - [ ] 编译期类型检查
  - [ ] 使用示例编译通过
- **代码审查**: 宏展开正确

### CP3-T14: 实现 trace_event! 宏
- **目标**: 瞬时事件记录宏
- **开发任务**:
  1. 宏签名: `trace_event!(name, key = value, ...)`
  2. 不返回 guard（瞬时事件，start = end）
- **Commit**: `feat(trace): implement trace_event! macro`
- **验收标准**:
  - [ ] 事件 span 的 start_tsc == end_tsc
  - [ ] 关联到当前 span 的 trace_id
- **代码审查**: 与 trace_span! 风格一致

### CP3-T15: 实现 TraceConfig
- **目标**: 运行时 trace 配置
- **开发任务**:
  1. 定义 `TraceConfig` 结构体
  2. 支持：全局开关、模块级开关、采样率
  3. 实现 `Tracer::configure()` 方法
- **Commit**: `feat(trace): add runtime trace configuration`
- **验收标准**:
  - [ ] 可全局关闭 trace
  - [ ] 可按模块关闭
  - [ ] 配置变更立即生效

### CP3-T16: 实现 TraceStats
- **目标**: 收集 trace 系统统计信息
- **开发任务**:
  1. 定义 `TraceStats` 结构体（总 span 数、活跃 span 数、buffer 使用率等）
  2. 实现 `Tracer::stats()` 方法
- **Commit**: `feat(trace): implement TraceStats collection`
- **验收标准**:
  - [ ] 统计数据实时准确
  - [ ] 包含性能相关指标

### CP3-T17: 实现 NullTracer — 空实现
- **目标**: 用于测试和禁用 trace 的空实现
- **开发任务**:
  1. 实现 `NullTracer`（所有方法为空操作）
  2. 用于其他模块的单元测试中
- **Commit**: `feat(trace): implement NullTracer for testing`
- **验收标准**:
  - [ ] 所有 Tracer trait 方法实现
  - [ ] 零开销（编译器可优化掉）
- **代码审查**: 实现完整

### CP3-T18 ~ CP3-T25: JSON 导出和串口输出

### CP3-T18: 实现 JSON 序列化 — Span
- **目标**: 将 Span 序列化为 JSON（不用 serde，手写 no_std 序列化）
- **Commit**: `feat(trace): implement JSON serialization for Span`
- **验收标准**:
  - [ ] 输出格式与 spec.md 定义一致
  - [ ] 不依赖 alloc（使用 Write trait 流式输出）

### CP3-T19: 实现 JSON 序列化 — TraceExport
- **目标**: 完整的 trace 导出文档序列化
- **Commit**: `feat(trace): implement full trace export JSON format`
- **验收标准**:
  - [ ] 包含 format 版本、tsc_frequency、spans 数组

### CP3-T20: 实现串口 trace 输出
- **目标**: 将 trace 数据实时输出到串口
- **Commit**: `feat(trace): implement serial trace output`
- **验收标准**:
  - [ ] trace 数据通过串口实时输出
  - [ ] 格式可被宿主机解析

### CP3-T21: 实现 trace 数据导出函数
- **目标**: 将 Ring Buffer 数据批量导出
- **Commit**: `feat(trace): implement bulk trace data export`
- **验收标准**:
  - [ ] 支持完整导出和过滤导出

### CP3-T22: 添加 TSC 频率校准
- **目标**: 将 TSC ticks 转换为纳秒/微秒
- **Commit**: `feat(trace): add TSC frequency calibration`
- **验收标准**:
  - [ ] 时间转换基本准确

### CP3-T23: 实现 trace 模块的初始化函数
- **目标**: `trace::init()` 初始化 Ring Buffer 和全局 Tracer
- **Commit**: `feat(trace): implement trace subsystem initialization`
- **验收标准**:
  - [ ] 初始化后 trace 宏可用

### CP3-T24: 集成 trace 到引导流程
- **目标**: 在 kernel_main 中为每个初始化步骤添加 trace span
- **Commit**: `feat(kernel): integrate trace spans into boot sequence`
- **验收标准**:
  - [ ] 引导过程产生完整的 trace 链
  - [ ] 串口可见 trace 输出

### CP3-T25: 实现 trace 输出的格式美化
- **目标**: 串口输出的 trace 数据可读性优化
- **Commit**: `feat(trace): format trace output for readability`
- **验收标准**:
  - [ ] 缩进显示层级关系
  - [ ] 时间显示为人类可读格式

### CP3-T26 ~ CP3-T40: 测试和边界情况

### CP3-T26: 测试 — Span 创建和属性
- **Commit**: `test(trace): add Span creation tests`
- **验收标准**: 覆盖正常创建和属性设置

### CP3-T27: 测试 — Ring Buffer 写入/读取
- **Commit**: `test(trace): add Ring Buffer read/write tests`
- **验收标准**: 覆盖空、满、覆盖场景

### CP3-T28: 测试 — Ring Buffer 边界条件
- **Commit**: `test(trace): add Ring Buffer boundary tests`
- **验收标准**: 覆盖容量边界、并发写入

### CP3-T29: 测试 — TraceContext 嵌套
- **Commit**: `test(trace): add TraceContext nesting tests`
- **验收标准**: 覆盖正常嵌套、最大深度

### CP3-T30: 测试 — SpanGuard RAII
- **Commit**: `test(trace): add SpanGuard RAII tests`
- **验收标准**: 正常退出和 early return 都正确

### CP3-T31: 测试 — SpanFilter 过滤
- **Commit**: `test(trace): add SpanFilter tests`
- **验收标准**: 各过滤条件独立和组合测试

### CP3-T32: 测试 — TraceConfig 配置
- **Commit**: `test(trace): add TraceConfig tests`
- **验收标准**: 开关生效、模块过滤生效

### CP3-T33: 测试 — JSON 序列化
- **Commit**: `test(trace): add JSON serialization tests`
- **验收标准**: 输出 JSON 格式正确

### CP3-T34: 测试 — NullTracer
- **Commit**: `test(trace): add NullTracer tests`
- **验收标准**: 所有操作不 panic

### CP3-T35: 测试 — 集成测试：引导 trace
- **Commit**: `test(kernel): add boot trace integration test`
- **验收标准**: QEMU 中引导产生 trace 数据

### CP3-T36 ~ CP3-T45: proc-macro 和优化

### CP3-T36: 实现 #[traced] 属性宏 — 基础框架
- **目标**: 过程宏自动包裹函数
- **Commit**: `feat(trace-macros): implement basic #[traced] attribute`
- **验收标准**:
  - [ ] 宏能正确包裹函数体
  - [ ] 编译通过

### CP3-T37: 实现 #[traced] — module 参数
- **目标**: 支持指定模块名
- **Commit**: `feat(trace-macros): add module parameter to #[traced]`
- **验收标准**:
  - [ ] `#[traced(module = "xxx")]` 语法正确

### CP3-T38: 实现 #[traced] — 返回值捕获
- **目标**: 在 span 中记录返回值类型
- **Commit**: `feat(trace-macros): capture return value in #[traced]`
- **验收标准**:
  - [ ] Result 类型的 Err 自动标记 span 为 Error

### CP3-T39: 测试 #[traced] 宏
- **Commit**: `test(trace-macros): add #[traced] attribute tests`
- **验收标准**: 编译和运行时行为正确

### CP3-T40: 优化 trace 性能 — 减少锁竞争
- **目标**: Ring Buffer 写入优化
- **Commit**: `refactor(trace): optimize Ring Buffer write performance`
- **验收标准**:
  - [ ] 写入路径无锁（纯原子操作）

### CP3-T41 ~ CP3-T52: 整合和收尾

### CP3-T41: 在 common crate 导出 trace 宏
- **目标**: 让所有 crate 都能使用 trace 宏
- **Commit**: `feat(common): re-export trace macros for workspace-wide use`
- **验收标准**: 任何 crate 都能 `use minios_common::trace_span!`

### CP3-T42: 代码审查 — trace 模块整体
- **目标**: 全面审查 trace 模块代码质量
- **Commit**: `refactor(trace): cleanup and code quality improvements` (如有改动)
- **验收标准**:
  - [ ] 无超过 50 行的函数
  - [ ] 无重复代码
  - [ ] 所有 pub API 有文档

### CP3-T43: 文档 — trace 模块 rustdoc
- **Commit**: `docs(trace): complete rustdoc for trace module`
- **验收标准**: `cargo doc` 无警告

### CP3-T44: 实现 Ring Buffer 清空功能
- **Commit**: `feat(trace): implement Ring Buffer clear`
- **验收标准**: 清空后统计归零

### CP3-T45: 确保 trace 在 panic 时也能输出
- **目标**: panic handler 中输出当前 trace 上下文
- **Commit**: `feat(kernel): output trace context in panic handler`
- **验收标准**: panic 时串口输出当前 trace_id 和 span_id

### CP3-T46: 代码审查触发 — 检查 HAL 是否需要重构
- **目标**: CP3 开发过程中 HAL 的变更是否引入技术债务
- **Commit**: `refactor(hal): ...` (如需要)
- **验收标准**: HAL 代码仍符合质量标准

### CP3-T47: 创建 ADR — Trace 引擎设计选择
- **Commit**: `docs(adr): ADR-005 trace engine design choices`
- **验收标准**: 记录 Ring Buffer vs 其他方案的权衡

### CP3-T48: 创建 ADR — 零分配 trace 策略
- **Commit**: `docs(adr): ADR-006 zero-allocation trace strategy`
- **验收标准**: 说明为什么不用 alloc

### CP3-T49: 更新风险登记簿
- **Commit**: `docs: update risk register for trace subsystem`
- **验收标准**: 记录 trace 性能风险

### CP3-T50: 验证完整 workspace 编译和测试
- **Commit**: N/A（验证任务）
- **验收标准**:
  - [ ] `cargo build --workspace` 通过
  - [ ] `cargo test --workspace` 通过
  - [ ] `cargo clippy --workspace` 零警告

### CP3-T51: 更新 CHANGELOG
- **Commit**: `docs: update CHANGELOG for CP3`
- **验收标准**: 记录所有 trace 相关变更

### CP3-T52: CP3 完成报告
- **Commit**: `docs: add CP3 completion report`
- **验收标准**: 包含性能基准数据

---

# CP4: 内存管理

> **目标**: 实现物理帧分配器、虚拟内存管理（4 级页表）和内核堆分配器
> **预计任务数**: 50
> **前置条件**: CP3 全部完成

## CP4 验收标准

- [ ] **AC-4.1**: 物理帧分配器能正确分配/释放帧，并追踪空闲帧数
- [ ] **AC-4.2**: 页表映射/取消映射/地址翻译正确工作
- [ ] **AC-4.3**: 内核堆分配器可用（`alloc::vec!`, `alloc::string::String` 等可用）
- [ ] **AC-4.4**: 所有内存操作有完整 trace span
- [ ] **AC-4.5**: 内存分配失败返回错误而非 panic
- [ ] **AC-4.6**: 单元测试覆盖正常/边界/错误路径
- [ ] **AC-4.7**: QEMU 中引导后能成功使用堆（`Vec::push` 等）
- [ ] **AC-4.8**: `meminfo` 可获取内存使用统计
- [ ] **AC-4.9**: 代码通过 clippy + fmt
- [ ] **AC-4.10**: 所有 unsafe 代码有 SAFETY 注释

## CP4 任务列表

### CP4-T01 ~ CP4-T12: 物理帧分配器

- **CP4-T01**: 解析 BootInfo 内存映射 → 识别可用物理帧范围
- **CP4-T02**: 实现 Bitmap 数据结构 → 位图存储帧使用状态
- **CP4-T03**: 实现 BitmapFrameAllocator::new() → 从内存映射初始化
- **CP4-T04**: 实现 allocate_frame() → 查找第一个空闲帧
- **CP4-T05**: 实现 deallocate_frame() → 标记帧为空闲
- **CP4-T06**: 实现 free_frames() / total_frames() → 统计方法
- **CP4-T07**: 添加 trace instrumentation → 每次 alloc/dealloc 记录 span
- **CP4-T08**: 实现线程安全 → SpinLock 保护
- **CP4-T09**: 测试 — 基本分配和释放
- **CP4-T10**: 测试 — 连续分配到耗尽
- **CP4-T11**: 测试 — 分配释放再分配
- **CP4-T12**: 测试 — 边界条件（空映射、重复释放）

### CP4-T13 ~ CP4-T25: 虚拟内存管理

- **CP4-T13**: 实现活动页表访问 → 读取 CR3 获取当前页表
- **CP4-T14**: 实现页表遍历 → 4 级页表地址翻译
- **CP4-T15**: 实现 translate_addr() → 虚拟地址到物理地址翻译
- **CP4-T16**: 实现 map_page() → 创建页表映射（按需创建中间页表）
- **CP4-T17**: 实现 unmap_page() → 取消映射并返回物理帧
- **CP4-T18**: 实现 PageFlags → 映射标志（WRITABLE, USER, NO_EXECUTE 等）
- **CP4-T19**: 实现 create_address_space() → 创建新的页表（用于进程）
- **CP4-T20**: 实现内核地址空间映射 → 物理内存直接映射
- **CP4-T21**: 添加 trace instrumentation → 页表操作的 span
- **CP4-T22**: 测试 — 映射和翻译
- **CP4-T23**: 测试 — 取消映射
- **CP4-T24**: 测试 — 多级页表创建
- **CP4-T25**: 测试 — 错误路径（已映射、未映射、对齐错误）

### CP4-T26 ~ CP4-T37: 内核堆分配器

- **CP4-T26**: 定义堆内存区域 → 虚拟地址范围和初始大小
- **CP4-T27**: 实现 LinkedListAllocator → 基本分配算法
- **CP4-T28**: 实现 allocate() → 首次适配查找空闲块
- **CP4-T29**: 实现 deallocate() → 合并相邻空闲块
- **CP4-T30**: 实现 GlobalAlloc trait → 接入 Rust 全局分配器
- **CP4-T31**: 实现堆初始化 → 映射堆页面并初始化分配器
- **CP4-T32**: 添加 trace instrumentation → 堆操作的 span
- **CP4-T33**: 实现 used_bytes() / free_bytes() → 堆统计
- **CP4-T34**: 测试 — Vec 分配
- **CP4-T35**: 测试 — String 分配
- **CP4-T36**: 测试 — Box 分配和释放
- **CP4-T37**: 测试 — 大量小对象分配

### CP4-T38 ~ CP4-T50: 集成和收尾

- **CP4-T38**: 集成到引导流程 → kernel_main 中初始化内存子系统
- **CP4-T39**: 实现 MemoryManager 门面 → 统一接口封装三个分配器
- **CP4-T40**: 验证 QEMU 中堆工作 → `Vec::push` 不 panic
- **CP4-T41**: 引导后显示内存统计 → 在 VGA 上输出可用/已用内存
- **CP4-T42**: 代码审查 — 内存模块整体质量
- **CP4-T43**: 代码审查 — unsafe 代码安全性
- **CP4-T44**: 文档 — 内存模块 rustdoc
- **CP4-T45**: 创建 ADR — 帧分配器算法选择
- **CP4-T46**: 创建 ADR — 堆分配器算法选择
- **CP4-T47**: 验证引导 trace 包含内存初始化 span
- **CP4-T48**: 修复 clippy 警告
- **CP4-T49**: 更新 CHANGELOG
- **CP4-T50**: CP4 完成报告

---

# CP5: 中断与异常处理

> **目标**: 实现 IDT、CPU 异常处理、硬件中断（时钟、键盘），键盘输入可在 VGA 上回显
> **预计任务数**: 45
> **前置条件**: CP4 全部完成

## CP5 验收标准

- [ ] **AC-5.1**: IDT 正确设置，CPU 异常有处理函数
- [ ] **AC-5.2**: Page Fault 处理函数能输出故障信息
- [ ] **AC-5.3**: Double Fault 使用独立栈，不会 Triple Fault
- [ ] **AC-5.4**: 时钟中断正常触发（每 10ms）
- [ ] **AC-5.5**: 键盘输入在 VGA 上回显
- [ ] **AC-5.6**: 所有中断处理有 trace span
- [ ] **AC-5.7**: 中断测试通过
- [ ] **AC-5.8**: 代码通过 clippy + fmt

## CP5 任务列表

- **CP5-T01**: 定义 InterruptFrame 结构体
- **CP5-T02**: 创建 IDT 静态实例
- **CP5-T03**: 实现 Division Error (0) 处理
- **CP5-T04**: 实现 Invalid Opcode (6) 处理
- **CP5-T05**: 实现 Double Fault (8) 处理（使用 IST）
- **CP5-T06**: 实现 General Protection Fault (13) 处理
- **CP5-T07**: 实现 Page Fault (14) 处理
- **CP5-T08**: 实现 IDT 加载函数 idt::init()
- **CP5-T09**: 添加异常处理的 trace span
- **CP5-T10**: 测试 — breakpoint 异常不 panic
- **CP5-T11**: 测试 — double fault 使用独立栈
- **CP5-T12**: 实现 Timer 中断处理函数 (IRQ 0)
- **CP5-T13**: 实现系统 tick 计数器
- **CP5-T14**: 添加 Timer trace span
- **CP5-T15**: 实现键盘中断处理函数 (IRQ 1)
- **CP5-T16**: 实现扫描码到 ASCII 转换
- **CP5-T17**: 实现 Shift/Ctrl/CapsLock 修饰键
- **CP5-T18**: 实现键盘事件队列
- **CP5-T19**: 实现键盘输入 VGA 回显
- **CP5-T20**: 添加键盘 trace span
- **CP5-T21**: 实现 InterruptManager trait
- **CP5-T22**: 实现中断注册机制（动态处理函数注册）
- **CP5-T23**: 实现中断统计（各 IRQ 触发次数）
- **CP5-T24**: 集成到引导流程
- **CP5-T25**: 启用中断（sti）
- **CP5-T26**: QEMU 验证 — 时钟中断工作
- **CP5-T27**: QEMU 验证 — 键盘输入回显
- **CP5-T28**: 测试 — Timer tick 计数增长
- **CP5-T29**: 测试 — 键盘事件队列
- **CP5-T30**: 测试 — 中断注册/注销
- **CP5-T31 ~ T35**: 代码审查、文档、异常处理优化
- **CP5-T36 ~ T40**: Page Fault 详细信息（地址、错误码）、中断嵌套保护
- **CP5-T41**: 创建 ADR — 中断处理策略
- **CP5-T42**: 验证 trace 链包含中断 span
- **CP5-T43**: 修复 clippy 警告
- **CP5-T44**: 更新 CHANGELOG
- **CP5-T45**: CP5 完成报告

---

# CP6: 进程管理与调度器

> **目标**: 实现进程/任务管理、上下文切换、MLFQ 调度器，多任务并发执行
> **预计任务数**: 55
> **前置条件**: CP5 全部完成

## CP6 验收标准

- [ ] **AC-6.1**: 可创建内核任务并独立运行
- [ ] **AC-6.2**: 上下文切换正确保存/恢复 CPU 寄存器
- [ ] **AC-6.3**: MLFQ 调度器按优先级调度任务
- [ ] **AC-6.4**: 时间片耗尽触发任务切换
- [ ] **AC-6.5**: 任务可主动 yield
- [ ] **AC-6.6**: 任务可正常退出
- [ ] **AC-6.7**: ps 命令可列出所有进程
- [ ] **AC-6.8**: 调度决策和上下文切换有完整 trace
- [ ] **AC-6.9**: 多个任务并发执行时系统稳定
- [ ] **AC-6.10**: 代码通过 clippy + fmt

## CP6 任务列表

- **CP6-T01**: 实现 PID 分配器（原子递增）
- **CP6-T02**: 定义 Process (PCB) 结构体
- **CP6-T03**: 定义 CpuContext 结构体（所有需要保存的寄存器）
- **CP6-T04**: 实现内核栈分配（每进程独立栈）
- **CP6-T05**: 实现 ProcessManager — 进程表（BTreeMap<Pid, Process>）
- **CP6-T06**: 实现 create_process() — 创建任务
- **CP6-T07**: 实现 exit_process() — 任务退出
- **CP6-T08**: 实现 current_pid() — 获取当前进程
- **CP6-T09**: 实现 list_processes() — 列出进程信息
- **CP6-T10**: 实现上下文切换（汇编） — switch_context()
- **CP6-T11**: 实现上下文切换的 Rust 封装
- **CP6-T12**: 添加进程管理 trace span
- **CP6-T13**: 测试 — 创建进程
- **CP6-T14**: 测试 — 进程退出
- **CP6-T15**: 测试 — PID 分配唯一性
- **CP6-T16**: 实现 MLFQ — 优先级队列数据结构
- **CP6-T17**: 实现 MLFQ — add_task()
- **CP6-T18**: 实现 MLFQ — tick() 逻辑
- **CP6-T19**: 实现 MLFQ — next_task() 选择
- **CP6-T20**: 实现 MLFQ — 时间片管理
- **CP6-T21**: 实现 MLFQ — 优先级降级（用完时间片）
- **CP6-T22**: 实现 MLFQ — 优先级提升（定期 boost）
- **CP6-T23**: 实现 MLFQ — yield_current()
- **CP6-T24**: 实现 MLFQ — block/unblock
- **CP6-T25**: 实现 MLFQ — stats() 统计
- **CP6-T26**: 添加调度器 trace span
- **CP6-T27**: 测试 — MLFQ 基本调度
- **CP6-T28**: 测试 — 优先级降级
- **CP6-T29**: 测试 — boost 防饥饿
- **CP6-T30**: 实现 idle 任务 — PID 0，hlt loop
- **CP6-T31**: 集成调度器到 Timer 中断
- **CP6-T32**: 实现 Timer 中断触发调度
- **CP6-T33**: 实现调度决策执行（在中断返回时切换）
- **CP6-T34**: 创建 init 进程 — PID 1
- **CP6-T35**: QEMU 验证 — 两个任务交替执行
- **CP6-T36**: QEMU 验证 — 多任务并发稳定性
- **CP6-T37**: 实现 kill_process()
- **CP6-T38**: 实现进程状态转换保护
- **CP6-T39**: 为上下文切换添加详细 trace
- **CP6-T40 ~ T45**: 代码审查、文档、异常安全性
- **CP6-T46**: 实现 ProcessInfo 格式化输出
- **CP6-T47**: 测试 — 进程状态机完整性
- **CP6-T48**: 测试 — kill 信号处理
- **CP6-T49 ~ T52**: ADR、风险更新、CHANGELOG
- **CP6-T53**: 验证 trace 链包含调度和切换 span
- **CP6-T54**: 修复 clippy 警告
- **CP6-T55**: CP6 完成报告

---

# CP7: 文件系统

> **目标**: 实现 VFS 层、RamFS、ProcFS、TraceFS、DevFS，支持基本文件操作
> **预计任务数**: 50
> **前置条件**: CP6 全部完成

## CP7 验收标准

- [ ] **AC-7.1**: VFS 支持 open/close/read/write/mkdir/readdir/stat
- [ ] **AC-7.2**: RamFS 正确存储和检索文件数据
- [ ] **AC-7.3**: 目录层级结构正确（创建/遍历/删除）
- [ ] **AC-7.4**: ProcFS 通过 `/proc/meminfo` 返回内存信息
- [ ] **AC-7.5**: TraceFS 通过 `/trace/current` 返回 trace 数据
- [ ] **AC-7.6**: DevFS 提供 `/dev/null` 和 `/dev/zero`
- [ ] **AC-7.7**: 文件描述符表正确管理
- [ ] **AC-7.8**: 所有文件操作有 trace span
- [ ] **AC-7.9**: 单元测试覆盖所有文件操作
- [ ] **AC-7.10**: 代码通过 clippy + fmt

## CP7 任务列表

- **CP7-T01 ~ T06**: VFS 层实现（路径解析、挂载点管理、操作分发）
- **CP7-T07**: 实现文件描述符表
- **CP7-T08 ~ T15**: RamFS 实现（Inode、文件创建/读/写、目录操作）
- **CP7-T16 ~ T20**: ProcFS 实现（meminfo、cpuinfo、uptime、进程状态）
- **CP7-T21 ~ T24**: TraceFS 实现（current、config、stats）
- **CP7-T25 ~ T28**: DevFS 实现（null、zero、console、serial）
- **CP7-T29**: 初始文件系统结构创建（/dev、/proc、/tmp、/trace、/etc）
- **CP7-T30 ~ T38**: 测试（VFS 操作、RamFS 存储、路径解析、边界条件）
- **CP7-T39**: 添加 trace instrumentation
- **CP7-T40**: 集成到引导流程
- **CP7-T41 ~ T45**: QEMU 验证、代码审查、文档
- **CP7-T46 ~ T50**: ADR、CHANGELOG、完成报告

---

# CP8: 系统调用与 IPC

> **目标**: 实现系统调用分发、基础 syscall、消息队列 IPC，并实现 trace 跨进程传播
> **预计任务数**: 50
> **前置条件**: CP7 全部完成

## CP8 验收标准

- [ ] **AC-8.1**: 系统调用通过 int 0x80 或 syscall 指令触发
- [ ] **AC-8.2**: 基础 I/O syscall (read/write/open/close) 工作
- [ ] **AC-8.3**: 进程 syscall (fork/exit/getpid/yield) 工作
- [ ] **AC-8.4**: 消息队列 IPC 可跨进程传递消息
- [ ] **AC-8.5**: IPC 消息携带 TraceContext，跨进程 trace 链路连贯
- [ ] **AC-8.6**: 系统调用有完整 trace span
- [ ] **AC-8.7**: 错误的 syscall 号返回 -ENOSYS
- [ ] **AC-8.8**: 单元测试覆盖所有 syscall
- [ ] **AC-8.9**: 代码通过 clippy + fmt
- [ ] **AC-8.10**: IPC 跨进程 trace 验证

## CP8 任务列表

- **CP8-T01 ~ T05**: Syscall 分发器（IDT 注册、参数传递、返回值）
- **CP8-T06 ~ T10**: I/O Syscall 实现（read、write、open、close、stat）
- **CP8-T11 ~ T15**: 进程 Syscall 实现（fork、exit、getpid、yield、waitpid）
- **CP8-T16 ~ T18**: 内存 Syscall 实现（mmap、munmap、meminfo）
- **CP8-T19 ~ T22**: Trace Syscall 实现（trace_dump、trace_config）
- **CP8-T23 ~ T28**: 消息队列 IPC 实现（创建/发送/接收/销毁）
- **CP8-T29 ~ T31**: 共享内存 IPC 实现（创建/附加/分离）
- **CP8-T32 ~ T34**: IPC TraceContext 传播实现
- **CP8-T35 ~ T42**: 测试（各类 syscall、IPC 通信、跨进程 trace）
- **CP8-T43 ~ T45**: 集成到引导流程、QEMU 验证
- **CP8-T46 ~ T50**: 代码审查、文档、ADR、CHANGELOG、完成报告

---

# CP9: Shell 交互终端

> **目标**: 实现完整的 Shell，包括命令解析、基础命令和 trace 查看命令
> **预计任务数**: 48
> **前置条件**: CP8 全部完成

## CP9 验收标准

- [ ] **AC-9.1**: Shell 启动后显示提示符，接受键盘输入
- [ ] **AC-9.2**: 基础命令工作（ls, cd, cat, mkdir, echo, ps, meminfo）
- [ ] **AC-9.3**: `trace list` 显示最近的 trace span
- [ ] **AC-9.4**: `trace tree` 树形显示 trace 链路
- [ ] **AC-9.5**: `trace export` 通过串口导出 JSON
- [ ] **AC-9.6**: `trace live` 实时显示 trace
- [ ] **AC-9.7**: 命令解析处理错误输入不 panic
- [ ] **AC-9.8**: Shell 的操作本身有 trace span
- [ ] **AC-9.9**: 帮助系统完整（help 命令）
- [ ] **AC-9.10**: 代码通过 clippy + fmt

## CP9 任务列表

- **CP9-T01 ~ T04**: 输入处理（键盘事件消费、行缓冲、退格/Ctrl+C）
- **CP9-T05 ~ T08**: 命令解析器（分词、命令查找、参数传递）
- **CP9-T09**: Shell 主循环
- **CP9-T10**: 提示符显示（pid + cwd）
- **CP9-T11 ~ T22**: 基础命令实现（help、echo、clear、ls、cd、pwd、cat、mkdir、touch、rm、write、ps）
- **CP9-T23 ~ T25**: 系统信息命令（meminfo、uptime、kill）
- **CP9-T26 ~ T32**: Trace 命令实现（trace list、trace tree、trace stats、trace filter、trace export、trace clear、trace live）
- **CP9-T33 ~ T38**: 测试（命令解析、各命令执行、错误处理）
- **CP9-T39**: 集成 Shell 为 init 进程
- **CP9-T40**: QEMU 验证 — Shell 交互
- **CP9-T41 ~ T45**: 代码审查、文档、边界处理
- **CP9-T46 ~ T48**: ADR、CHANGELOG、完成报告

---

# CP10: Trace 可视化与集成测试

> **目标**: 实现宿主机 Web 可视化工具，完成端到端集成测试和系统打磨
> **预计任务数**: 50
> **前置条件**: CP9 全部完成

## CP10 验收标准

- [ ] **AC-10.1**: Web 工具可加载 JSON trace 文件
- [ ] **AC-10.2**: 瀑布图正确展示 span 时间线和层级关系
- [ ] **AC-10.3**: 火焰图正确展示调用栈
- [ ] **AC-10.4**: 时间线按进程分行展示
- [ ] **AC-10.5**: 模块依赖图展示模块间调用关系
- [ ] **AC-10.6**: 交互功能工作（缩放、悬停、搜索）
- [ ] **AC-10.7**: 端到端流程：引导 → Shell → 执行命令 → 导出 trace → 可视化
- [ ] **AC-10.8**: 性能目标达成（trace 开销 < 500ns）
- [ ] **AC-10.9**: 系统稳定运行 > 5 分钟无 crash
- [ ] **AC-10.10**: 最终演示视频/截图可生成

## CP10 任务列表

### Web 可视化工具 (CP10-T01 ~ T25)

- **CP10-T01**: 创建 HTML 页面框架
- **CP10-T02**: 实现暗色主题 CSS
- **CP10-T03**: 实现文件加载（拖拽 + 文件选择）
- **CP10-T04**: 实现 JSON trace 解析器
- **CP10-T05**: 实现 span 树构建（parent-child 关系重建）
- **CP10-T06 ~ T10**: 瀑布图实现（Canvas 绘制、时间轴、颜色编码、层级缩进、hover 信息）
- **CP10-T11 ~ T14**: 火焰图实现（宽度=时间比例、堆叠、交互）
- **CP10-T15 ~ T17**: 时间线实现（多进程行、调度点标记）
- **CP10-T18 ~ T20**: 模块依赖图实现（SVG、边粗细=频率）
- **CP10-T21**: 实现搜索/过滤功能
- **CP10-T22**: 实现缩放/平移控制
- **CP10-T23**: 创建示例 trace 数据
- **CP10-T24**: 测试 — 各视图渲染正确
- **CP10-T25**: 测试 — 大数据量性能

### 集成测试和打磨 (CP10-T26 ~ T50)

- **CP10-T26**: 端到端测试 — 引导到 Shell
- **CP10-T27**: 端到端测试 — Shell 命令执行
- **CP10-T28**: 端到端测试 — 文件系统操作
- **CP10-T29**: 端到端测试 — 进程创建和调度
- **CP10-T30**: 端到端测试 — trace 导出和可视化
- **CP10-T31**: 端到端测试 — IPC 跨进程 trace
- **CP10-T32**: 稳定性测试 — 长时间运行
- **CP10-T33**: 性能测试 — trace 开销测量
- **CP10-T34**: 性能测试 — 上下文切换时间
- **CP10-T35**: 性能测试 — 内存分配速度
- **CP10-T36 ~ T38**: Bug 修复（根据测试发现）
- **CP10-T39**: 代码质量 — 全项目 clippy 审查
- **CP10-T40**: 代码质量 — 全项目 unsafe 审查
- **CP10-T41**: 代码质量 — 全项目文档完整性
- **CP10-T42**: 代码质量 — 全项目重复代码消除
- **CP10-T43**: 创建 capture-trace.sh 脚本
- **CP10-T44**: 更新 README — 完整使用指南
- **CP10-T45**: 更新 AGENTS.md — 最终开发指引
- **CP10-T46**: 创建最终 ADR — 系统总结
- **CP10-T47**: 更新风险登记簿 — 关闭已解决风险
- **CP10-T48**: 完成 CHANGELOG
- **CP10-T49**: 生成最终演示材料
- **CP10-T50**: CP10 完成报告（项目总结）

---

## 第四部分：风险预判

### 已识别风险

| ID | 风险 | 概率 | 影响 | 缓解策略 | 触发检查点 |
|----|------|------|------|---------|-----------|
| R01 | bootloader crate API 不兼容 | 中 | 高 | 锁定版本，参考 os.phil-opp.com 最新教程 | CP2 |
| R02 | 上下文切换汇编实现困难 | 高 | 高 | 参考成熟实现（xv6, BlogOS），充分测试 | CP6 |
| R03 | Trace Ring Buffer 并发安全 | 中 | 高 | 使用原子操作，大量并发测试 | CP3 |
| R04 | 内存管理导致系统不稳定 | 中 | 高 | 渐进式测试，每步验证 | CP4 |
| R05 | MLFQ 调度器饥饿问题 | 低 | 中 | 实现 boost 机制，压力测试 | CP6 |
| R06 | QEMU 与裸机行为差异 | 低 | 中 | 以 QEMU 为准，记录差异 | 全程 |
| R07 | proc-macro crate 与 no_std 交互 | 中 | 中 | proc-macro 运行在宿主机，小心设计接口 | CP3 |
| R08 | 堆分配器碎片化 | 中 | 低 | 初期用简单算法，后续可替换 | CP4 |
| R09 | VGA 文本模式限制 | 低 | 低 | trace 查看器使用串口辅助，Web 工具为主 | CP9 |
| R10 | 跨进程 trace 传播复杂度 | 中 | 中 | IPC 消息天然携带上下文，设计时预留 | CP8 |

### 需要人工决策的点（预判）

| ID | 决策点 | 检查点 | 默认选择 | 说明 |
|----|--------|--------|---------|------|
| D01 | bootloader 版本选择 | CP1 | 最新稳定版 | 需确认 API 兼容性 |
| D02 | BIOS 还是 UEFI 引导 | CP2 | BIOS（bootloader crate 同时支持） | UEFI 更现代但复杂 |
| D03 | 帧分配器算法 | CP4 | Bitmap（简单可靠） | Buddy 性能更好但复杂 |
| D04 | 堆分配器算法 | CP4 | Linked List（简单） | Slab 性能更好但复杂 |
| D05 | 用户态隔离范围 | CP6 | 仅内核态任务（简化） | 完整用户态需额外大量工作 |
| D06 | Shell 内置 vs 外部命令 | CP9 | 全部内置 | 外部命令需 ELF 加载器 |

---

## 第五部分：进度总览表

| 检查点 | 名称 | 任务数 | 状态 | 完成日期 |
|--------|------|--------|------|---------|
| CP1 | 项目脚手架与构建系统 | 48 | ⬜ 未开始 | - |
| CP2 | HAL 硬件抽象层 + 引导 | 52 | ⬜ 未开始 | - |
| CP3 | Trace 引擎 | 52 | ⬜ 未开始 | - |
| CP4 | 内存管理 | 50 | ⬜ 未开始 | - |
| CP5 | 中断与异常处理 | 45 | ⬜ 未开始 | - |
| CP6 | 进程管理与调度器 | 55 | ⬜ 未开始 | - |
| CP7 | 文件系统 | 50 | ⬜ 未开始 | - |
| CP8 | 系统调用与 IPC | 50 | ⬜ 未开始 | - |
| CP9 | Shell 交互终端 | 48 | ⬜ 未开始 | - |
| CP10 | Trace 可视化与集成测试 | 50 | ⬜ 未开始 | - |
| **合计** | | **500** | | |

---

*计划版本: 1.0*
*最后更新: 2026-02-25*
*关联规格: spec.md v1.0*
