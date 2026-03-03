---
name: create-issue
description: Create GitHub issues for the RobustMQ project. Use when the user asks to create an issue, file a bug, propose a feature, or track a task.
---

# Create GitHub Issue

Create GitHub issues for the RobustMQ repository using the project's issue script and templates.

## When to Use

- User asks to create a GitHub issue
- User wants to file a bug report, feature request, or enhancement
- User wants to track a subtask of an existing issue

## Instructions

### 1. Determine Issue Type

Ask the user or infer from context which template to use:

| Template | Title Prefix | Labels | When |
|----------|-------------|--------|------|
| Feature Request | `[Feat]` | `kind:feat` | Brand new feature |
| Enhancement | `[Style/Refactor/Performance/Docs/Test/Build/CI/Chore]` | `enhancement` | Improve existing functionality |
| Bug Report | `[BUG]` | `kind:bug` | Something is broken |
| Subtask | `[RBIP-*/MINOR][...][Subtask]` | `kind:subtask` | Subtask of a parent issue |

Template definitions are in `.github/ISSUE_TEMPLATE/`.

### 2. Write the Issue

- **Always write in English**, even if the user describes the issue in Chinese.
- Follow the body structure of the matching template (see below).
- Be concise and technical.

### 3. Execute

- **Never create temporary files.** Pass the body inline.
- Ask the user for their `GITHUB_TOKEN` if not already set in the environment.
- Use `scripts/create-issue.sh`:

```bash
export GITHUB_TOKEN=<user-provided token>

./scripts/create-issue.sh \
  --title "<title with correct prefix>" \
  --body "$(cat <<'EOF'
<body content>
EOF
)" \
  --labels "<labels>"
```

### 4. Report Back

Return the created issue URL to the user.

## Body Structure by Template

### Feature Request (`kind:feat`)

```
## What problem does the new feature solve?
<describe the problem and why it matters>

## What does the feature do?
<high-level overview>

## Implementation challenges
<optional: technical details, design considerations>
```

### Enhancement (`enhancement`)

```
## Type
<one of: Style / Refactor / Performance / Doc / Test / Build / CI / Chore>

## What does the enhancement do?
<description of the improvement>

## Implementation challenges
<optional: technical details>
```

### Bug Report (`kind:bug`)

```
## Bug type
<one of: Configuration / Crash / Data corruption / Incorrect result / Locking issue / Performance issue / Unexpected error / Other>

## Affected subsystem
<one of: Standalone mode / Frontend / Datanode / Meta / Other>

## Minimal reproduce step
<steps to reproduce>

## Expected behavior
<what should happen>

## Actual behavior
<what actually happens>

## Environment
<OS, version, architecture>

## Relevant logs
<optional: log output or stack trace>
```

### Subtask (`kind:subtask`)

```
## Describe the subtask
<clear description of the subtask>

## Parent issue
<link to the parent issue>
```
