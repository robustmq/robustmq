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

### 手动调用

在 Cursor Agent 聊天框或 Claude CLI 中输入斜杠命令：

```
/create-issue
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
