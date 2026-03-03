# AI Skills 使用指南

RobustMQ 项目内置了一组 [Agent Skills](https://agentskills.io/)，用于标准化日常开发中的重复性工作流。Skills 是一种开放标准，可以在 Cursor、Claude CLI（Claude Code）等 AI 工具中使用。

## 什么是 Skills

Skills 是存放在项目仓库中的结构化指令包。它告诉 AI 助手在执行特定任务时应该遵循的规则、使用的工具和输出格式。

优势：
- **团队一致性**：所有人用同一套工作流，不需要每次手动交代
- **版本可控**：Skills 随代码一起 Git 管理
- **跨工具通用**：Cursor 和 Claude CLI 都支持

## 目录结构

```
.claude/skills/
├── robustmq-metrics/
│   └── SKILL.md
├── connector-delivery/
│   └── SKILL.md
└── create-issue/
    └── SKILL.md
```

Skills 存放在 `.claude/skills/` 目录下。Cursor 和 Claude CLI 都会自动发现并加载这些 Skills。

## 使用方式

### 自动触发

直接在对话中描述你的需求，AI 会根据 Skill 的 `description` 自动匹配：

```
帮我创建一个 issue，给 meta storage 加上 per-key 版本号
```

自动触发是否成功，取决于 `description` 是否同时包含：
- **做什么（WHAT）**：能力边界和目标
- **何时使用（WHEN）**：触发关键词和场景

例如：
- “实现新 connector、补齐 admin parse、更新 docs” 这类描述，会触发 `connector-delivery`
- “创建 bug/feature issue” 这类描述，会触发 `create-issue`

### 手动调用

在 Cursor Agent 聊天框或 Claude CLI 中输入斜杠命令：

```
/create-issue
```

或：

```
/connector-delivery
```

或：

```
/robustmq-metrics
```

## 当前可用的 Skills

### create-issue

**用途**：为 RobustMQ 项目创建 GitHub Issue。

**触发条件**：当你要求创建 issue、提 bug、提 feature 等。

**自动执行的规则**：
- 使用英文撰写 issue 标题和内容
- 根据内容自动选择正确的 issue 模板（Feature / Enhancement / Bug / Subtask）
- 使用 `scripts/create-issue.sh` 脚本创建，不生成临时文件
- 自动添加正确的标题前缀和 labels

**支持的 Issue 类型**：

| 类型 | 标题前缀 | 标签 |
|------|---------|------|
| 新功能 | `[Feat]` | `kind:feat` |
| 改进 | `[Style/Refactor/Performance/Docs/Test/Build/CI/Chore]` | `enhancement` |
| Bug | `[BUG]` | `kind:bug` |
| 子任务 | `[RBIP-*/MINOR][...][Subtask]` | `kind:subtask` |

**前置条件**：需要提供 GitHub Token（首次使用时 AI 会主动询问）。

**使用示例**：

```
帮我创建一个 feature issue，内容是给 meta storage 中的数据存储加上 per-key 版本号，
用来解决分布式环境下数据写入顺序乱序的问题。
```

---

### connector-delivery

**用途**：标准化新增 Connector 的全流程交付（metadata、runtime、admin、文档、校验）。

**触发条件**：当你要求“新增/实现/支持某个 connector”（如 `s3`、`kinesis`、`webhook`）时。

**自动执行的规则**：
- 先补 `metadata-struct` 配置与校验
- 再补 `ConnectorType` 注册与 `FromStr` 分支
- 接入 runtime 模块与 `core.rs` dispatch
- 接入 admin 侧 `validate_connector_type` 与 `parse_connector_type`
- 同步 zh/en API 与 Bridge Overview 文档
- 运行 `cargo check -p metadata-struct -p connector -p admin-server`

**使用示例**：

```
实现 AWS S3 connector，按现有模式接入。
```

---

### robustmq-metrics

**用途**：为 RobustMQ 设计并落地“精炼但覆盖核心链路”的指标体系（通用规范，不绑定某个业务模块）。

**触发条件**：当你要求“加 metrics / 完善可观测性 / 补 Grafana 面板”时。

**自动执行的规则**：
- 指标设计遵循“够用覆盖链路，不贪多”
- 处理链路必须覆盖**次数 + 耗时**
- 指标定义统一放在 `src/common/metrics`（按模块落位）
- labels 默认使用低基数、稳定维度；业务名/实体名标签需评估基数后再加
- 判断是否需要同步 `grafana/robustmq-broker.json`，若是核心运维指标则补面板

**使用示例**：

```
完善 connector 指标，补齐 retry、dlq、commit/read failure，并更新 Grafana 面板。
```

## 添加新的 Skill

如果你想添加新的 Skill，按照以下步骤操作：

1. 在 `.claude/skills/` 下创建一个新目录
2. 在目录中创建 `SKILL.md` 文件，包含 YAML frontmatter 和 Markdown 指令
3. 提交到仓库

`SKILL.md` 基本格式：

```markdown
---
name: my-skill
description: 描述这个 Skill 做什么，以及什么时候应该使用它。
---

# My Skill

详细指令...
```

更多信息请参考 [Agent Skills 规范](https://agentskills.io/)。

## 编写高质量 Skill（实践规范）

### 1) 命名规范

- `name` 使用小写 + 连字符（如 `connector-delivery`）
- 避免泛化命名（如 `helper`、`utils`）

### 2) description 规范（最关键）

`description` 决定能否被自动触发，建议写法：

1. 先写能力：这个 Skill 能做什么
2. 再写触发条件：用户说什么时应该启用
3. 加入关键词：任务名、文件名、场景词

示例（推荐）：

```yaml
description: Implements new RobustMQ connectors end-to-end. Use when the user asks to add, implement, or support a connector type such as s3, webhook, opentsdb, or kafka-compatible targets.
```

### 3) 内容结构建议

建议在 `SKILL.md` 中固定包含：
- `Purpose`：目标与边界
- `Input Contract`：最少输入
- `Implementation Workflow`：步骤化流程
- `Validation and Checks`：验收命令
- `Guardrails`：禁忌与风险

### 4) 长度与颗粒度

- 一个 Skill 只做一类任务（单一职责）
- 文档尽量短而可执行，优先 checklist
- 复杂背景放外部参考文档，不堆在 SKILL.md

## 调试与验证 Skill

当你感觉 Skill 没触发或执行不稳定，按这个顺序检查：

1. **description 是否可触发**：是否写清 WHAT + WHEN
2. **任务语句是否命中关键词**：用户表达是否足够具体
3. **是否需要手动调用**：先用 `/skill-name` 验证链路
4. **输出是否符合模板**：是否按 Skill 约定结构返回

建议每个 Skill 提供至少 2 条“可复制示例输入”，用于回归测试。

## 评审清单（提交前）

- [ ] Skill 放在 `.claude/skills/<skill-name>/SKILL.md`
- [ ] `name` 合法（小写 + 连字符）
- [ ] `description` 包含 WHAT + WHEN + 触发词
- [ ] 指令步骤明确、可执行、无歧义
- [ ] 包含验证命令或验收标准
- [ ] 至少有 1 个真实示例输入
