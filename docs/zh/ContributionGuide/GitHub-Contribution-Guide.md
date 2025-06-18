# GitHub 贡献指南

本指南定义了 ISSUE 和 Pull Request（以下简称 PR）的创建流程及规范，以确保协作高效且规范统一。

## 一、创建 ISSUE

### ISSUE 类型

请根据实际内容正确选择 ISSUE 类型：

- **feat**：新增功能或特性
- **bug**：Bug 问题修复
- **test**：新增或修改测试用例
- **refactor**：代码重构（无新功能或Bug修复）
- **docs**：文档或代码注释的更新
- **perf**：性能优化（需提供优化依据或测试报告）
- **build**：构建系统或依赖项的变更
- **ci**：持续集成配置的修改
- **style**：代码格式修改（不影响功能）
- **chore**：杂项，如开发工具或项目配置变更

### 用户与贡献者创建 ISSUE

用户和贡献者可直接使用 GitHub Issue 模板创建 ISSUE：

- **BugReport**：用于报告问题或缺陷
- **FeatureRequest**：用于请求新的功能特性
- **Enhancement**：用于对现有功能的增强需求

![image](../../images/GithubContributionGuide-1.png)

### Committer 创建 ISSUE

- 对于小型任务项目，参照用户和贡献者流程，创建 ISSUE 并关联 PR。

- 对于大型或复杂任务，建议创建一个**总任务（Umbrella）** ISSUE，并在其中规划和管理子任务：

![image](../../images/GithubContributionGuide-2.png)

- **总任务（Umbrella）** 标题命名规范：

  - `[RBIP-*]`：特性或新功能添加，如 `RBIP-09`。

    ![image](../../images/GithubContributionGuide-3.png)

  - `[MINOR]`：小功能或优化点，如 `[MINOR] 更新登录提示`。

    ![image](../../images/GithubContributionGuide-4.png)

  - **子任务（Subtask）** 创建规范：

    - 标题格式：`[RBIP-*/MINOR][类型][Subtask] 具体任务描述`
    - 示例：`[RBIP-09][Docs][Subtask] 完善登录功能说明文档`

    ![image](../../images/GithubContributionGuide-5.png)

## 二、创建 Pull Request

### PR 关联 ISSUE 规则

每个 PR 应尽可能关联对应的 ISSUE。关联方式是在 PR 内容中添加：

```
close #ISSUE_NUMBER
```

其中 `close` 和 `#` 为固定前缀，`ISSUE_NUMBER` 为对应的 ISSUE 编号。

![image](../../images/doc-image11.png)

#297，#292 就是对应的 ISSUE 编号。

- 示例：若 PR 修复 ISSUE #297，则 PR 描述中需明确注明：

```
close #297
```

- PR 合并后，该 ISSUE 将自动关闭。

  - PR：
    ![image](../../images/doc-image12.png)

  - ISSUE：
    ![image](../../images/doc-image13.png)

### PR 标题格式要求

每个 PR 的标题必须遵循以下规范：

```
类型: 简要描述
```

类型包括以下选项：

- **feat**：新增功能
- **fix**：Bug 修复
- **docs**：文档更新
- **style**：代码风格或格式变动
- **refactor**：代码重构
- **perf**：性能优化
- **test**：测试代码变更
- **chore**：杂项（非业务代码变动）
- **build**：构建系统或依赖变动
- **ci**：CI 配置或脚本变动
- **revert**：代码回滚

示例：

```
feat: 支持 RocksDB 存储引擎
fix: 修复登录跳转问题
```

### PR 提交检查

在提交 PR 前，建议执行以下检查，避免因格式或 License 问题导致的 PR 提交失败：

- **License 检查**：

```bash
cargo install hawkeye

# 检测缺失 License 的文件
hawkeye check

# 自动为文件添加 License
hawkeye format
```

- **标题检查**：务必确认 PR 标题符合上述格式规范。

### 签名

签名是代码说明末尾的一行简单文字，用于证明该代码是您编写的，或者您有权将其作为开源代码发布。

您只需在每个 git 提交消息中添加一行:

```text
Signed-off-by: your_name <your_email>
```

您可以在创建 git 提交时通过 `git commit -s` 添加签名。

如果您希望自动执行此操作，您可以设置一些别名:

```bash
# 这将在提交时自动添加签名
git config --add alias.c "commit -s"
```

提交的 commit 消息示例:
```text
feat: a good start!

Signed-off-by: robustmq <robustmq@outlook.com>
```

## 三、常见问题与解决方案

### 1. License 检测失败

- 使用上述 `hawkeye` 工具检查并自动修复 License 问题。

### 2. 标题格式错误

- 确认标题格式符合规定，如：`feat: 新增用户登录功能`
- 不得出现未规范的前缀或无前缀的情况。

更多细节，可参阅[提交 PR 的完整示例](./Pull-Request-Example.md)。

### 3. 修复 DCO 检查失败错误

如果您的 PR 未通过 DCO 检查，则需要修复 PR 中的整个提交历史记录。

我们建议你可以将 PR 中的 commit 压缩为单个，并按照 [签名](#签名) 流程附带签名，然后进行强制提交。

例如，您的 PR 中存在 3 个 commit:

```bash
git rebase -i HEAD^3
(interactive squash + sign off append)
git push origin -f
```

请注意，这种方式会导致多个 commit 记录压缩为一个导致 review 困难，仅用于修复 DCO 检查。

最佳实践是在每个提交中都附带签名。

---
