# GitHub Issue 创建脚本使用指南

通用的 GitHub Issue 创建脚本，支持多种方式创建 issue。

## 前置要求

### 1. 设置 GitHub Token

创建 GitHub Personal Access Token：
1. 访问：https://github.com/settings/tokens
2. 点击 "Generate new token (classic)"
3. 选择权限：`repo` (Full control of private repositories)
4. 复制生成的 token

设置环境变量：
```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
```

可以将其添加到 `~/.bashrc` 或 `~/.zshrc` 中永久保存：
```bash
echo 'export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx' >> ~/.bashrc
source ~/.bashrc
```

### 2. 安装依赖

确保系统已安装 `jq`（用于 JSON 处理）：

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# CentOS/RHEL
sudo yum install jq
```

## 使用方式

### 方式 1：命令行参数（推荐）

#### 基本用法 - 内联文本
```bash
./scripts/create-issue.sh \
  --title "Fix: Memory leak in retain message cache" \
  --body "Memory grows indefinitely when retain messages expire" \
  --labels "bug,mqtt,priority:high"
```

#### 从 Markdown 文件读取内容
```bash
./scripts/create-issue.sh \
  --title "Feature: Multi-tenant support via User Properties" \
  --body-file docs/issues/multi-tenant-feature.md \
  --labels "enhancement,mqtt,mqtt5,multi-tenant"
```

#### 简短参数
```bash
./scripts/create-issue.sh \
  -t "Issue Title" \
  -f docs/issues/my-feature.md \
  -l "bug,enhancement"
```

#### 自定义仓库
```bash
./scripts/create-issue.sh \
  -t "Issue Title" \
  -b "Issue body" \
  -l "bug" \
  -r "myorg/myrepo"
```

### 方式 2：交互式模式

```bash
./scripts/create-issue.sh --interactive
```

脚本会依次提示输入：
1. Repository（默认：robustmq/robustmq）
2. Issue Title
3. Issue Body（可以选择文件或直接输入）
4. Labels（逗号分隔）

### 方式 3：查看帮助

```bash
./scripts/create-issue.sh --help
```

## 参数说明

| 参数 | 简写 | 说明 | 必填 |
|------|------|------|------|
| `--title` | `-t` | Issue 标题 | 是* |
| `--body` | `-b` | Issue 正文（内联文本） | 是** |
| `--body-file` | `-f` | Issue 正文（从文件读取） | 是** |
| `--labels` | `-l` | 标签（逗号分隔） | 否 |
| `--repo` | `-r` | 目标仓库（格式：owner/repo） | 否 |
| `--interactive` | `-i` | 交互式模式 | 否 |
| `--help` | `-h` | 显示帮助信息 | 否 |

\* 除非使用 `--interactive` 模式  
\*\* `--body` 和 `--body-file` 二选一

## 使用示例

### 示例 1：报告 Bug

```bash
./scripts/create-issue.sh \
  -t "[BUG] Connection leak in MQTT broker" \
  -b "When clients disconnect ungracefully, connections are not properly cleaned up, leading to resource exhaustion." \
  -l "bug,mqtt,priority:high"
```

### 示例 2：功能需求（从文件）

先创建一个 Markdown 文件 `docs/issues/my-feature.md`，然后：

```bash
./scripts/create-issue.sh \
  -t "Feature: Add support for MQTT bridge" \
  -f docs/issues/my-feature.md \
  -l "enhancement,mqtt,feature"
```

### 示例 3：多租户功能（示例文件已提供）

```bash
./scripts/create-issue.sh \
  -t "Feature: 通过 MQTT 5.0 User Properties 实现多租户功能" \
  -f docs/issues/multi-tenant-feature.md \
  -l "enhancement,mqtt,mqtt5,multi-tenant"
```

### 示例 4：文档改进

```bash
./scripts/create-issue.sh \
  -t "Docs: Add troubleshooting guide for common errors" \
  -b "Users frequently ask about common setup issues. We should add a comprehensive troubleshooting guide." \
  -l "documentation,good-first-issue"
```

### 示例 5：性能优化

```bash
./scripts/create-issue.sh \
  -t "Perf: Optimize topic matching algorithm" \
  -b "Current topic matching with wildcards is O(n). Consider using trie or other data structures for O(log n) performance." \
  -l "performance,optimization"
```

## 常用标签

推荐的标签分类：

**类型：**
- `bug` - Bug 修复
- `enhancement` - 功能增强
- `feature` - 新功能
- `documentation` - 文档相关
- `performance` - 性能优化
- `refactor` - 代码重构
- `test` - 测试相关

**优先级：**
- `priority:critical` - 紧急
- `priority:high` - 高优先级
- `priority:medium` - 中优先级
- `priority:low` - 低优先级

**模块：**
- `mqtt` - MQTT 协议相关
- `mqtt5` - MQTT 5.0 特性
- `storage` - 存储引擎
- `cluster` - 集群功能
- `security` - 安全相关

**状态：**
- `good-first-issue` - 适合新手
- `help-wanted` - 需要帮助
- `wontfix` - 不会修复
- `duplicate` - 重复 issue

## Markdown 文件模板

创建 issue 内容的 Markdown 文件模板：

### Bug 报告模板

```markdown
## 问题描述

简要描述遇到的问题。

## 复现步骤

1. 启动 MQTT Broker
2. 连接客户端
3. 发布消息到 topic `test/topic`
4. 观察到错误

## 预期行为

应该正常发布消息并投递给订阅者。

## 实际行为

消息丢失，订阅者未收到。

## 环境信息

- RobustMQ 版本：v1.0.0
- 操作系统：Ubuntu 22.04
- Rust 版本：1.75.0

## 日志输出

\`\`\`
[ERROR] Connection error: ...
\`\`\`

## 附加信息

其他相关信息。
```

### 功能需求模板

```markdown
## 功能需求

简要描述需要的功能。

## 背景和动机

为什么需要这个功能？解决什么问题？

## 功能设计

### 用户接口

如何使用这个功能？API 设计？

### 实现方案

技术实现思路。

### 配置示例

\`\`\`toml
[feature_config]
enabled = true
\`\`\`

## 预期效果

实现后的效果和收益。

## 实现步骤

- [ ] 步骤 1
- [ ] 步骤 2
- [ ] 步骤 3
```

## 故障排查

### Token 无效

```
Error: Bad credentials (401)
```

**解决方法：**
1. 检查 token 是否正确
2. 确认 token 有 `repo` 权限
3. Token 可能已过期，重新生成

### jq 命令未找到

```
./scripts/create-issue.sh: line X: jq: command not found
```

**解决方法：**
安装 jq：`brew install jq` (macOS) 或 `sudo apt-get install jq` (Linux)

### 文件未找到

```
Error: Body file not found: docs/issues/my-file.md
```

**解决方法：**
检查文件路径是否正确，使用相对路径或绝对路径。

### 权限被拒绝

```
permission denied: ./scripts/create-issue.sh
```

**解决方法：**
添加执行权限：`chmod +x scripts/create-issue.sh`

## 高级用法

### 批量创建 Issues

创建一个脚本来批量创建 issues：

```bash
#!/bin/bash

issues=(
  "Bug: Fix memory leak|docs/issues/bug1.md|bug,mqtt"
  "Feature: Add metrics|docs/issues/feature1.md|enhancement"
)

for issue in "${issues[@]}"; do
  IFS='|' read -r title file labels <<< "$issue"
  ./scripts/create-issue.sh -t "$title" -f "$file" -l "$labels"
  sleep 2  # 避免触发 API 限流
done
```

### 从模板生成 Issue

```bash
# 定义变量
TITLE="Fix: $BUG_NAME"
BODY_FILE="templates/bug-template.md"

# 替换模板中的变量
sed "s/{{BUG_NAME}}/$BUG_NAME/g" "$BODY_FILE" > /tmp/issue.md

# 创建 issue
./scripts/create-issue.sh -t "$TITLE" -f /tmp/issue.md -l "bug"
```

## 贡献指南

如果你想改进这个脚本，欢迎提交 PR！

改进方向：
- 支持更多的 GitHub issue 属性（assignees, milestone, projects）
- 支持草稿模式
- 支持 issue 模板
- 增加错误重试机制
- 添加更多的验证逻辑
