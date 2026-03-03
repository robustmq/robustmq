# AI Skills Guide

RobustMQ ships with a set of built-in [Agent Skills](https://agentskills.io/) that standardize repetitive development workflows. Skills are an open standard supported by Cursor, Claude CLI (Claude Code), and other AI tools.

## What are Skills

Skills are structured instruction packages stored in the repository. They tell AI assistants which rules to follow, which tools to use, and what output format to produce when performing specific tasks.

Benefits:
- **Team consistency**: Everyone uses the same workflow without repeating instructions
- **Version controlled**: Skills are tracked in Git alongside the code
- **Cross-tool**: Both Cursor and Claude CLI support them

## Directory Structure

```
.claude/skills/
└── create-issue/
    └── SKILL.md
```

Skills are stored in the `.claude/skills/` directory. Both Cursor and Claude CLI automatically discover and load these skills.

## Usage

### Automatic Invocation

Describe your intent in a conversation, and the AI will match the appropriate skill based on its `description`:

```
Create an issue to add per-key versioning to meta storage
```

### Manual Invocation

Type a slash command in the Cursor Agent chat or Claude CLI:

```
/create-issue
```

## Available Skills

### create-issue

**Purpose**: Create GitHub issues for the RobustMQ project.

**Trigger**: When you ask to create an issue, file a bug, propose a feature, etc.

**Automated rules**:
- Issue title and body are always written in English
- Automatically selects the correct issue template (Feature / Enhancement / Bug / Subtask)
- Uses `scripts/create-issue.sh` to create the issue, no temporary files
- Adds the correct title prefix and labels

**Supported issue types**:

| Type | Title Prefix | Labels |
|------|-------------|--------|
| Feature Request | `[Feat]` | `kind:feat` |
| Enhancement | `[Style/Refactor/Performance/Docs/Test/Build/CI/Chore]` | `enhancement` |
| Bug Report | `[BUG]` | `kind:bug` |
| Subtask | `[RBIP-*/MINOR][...][Subtask]` | `kind:subtask` |

**Prerequisites**: A GitHub Token is required (the AI will ask for it on first use).

**Example**:

```
Create a feature issue for adding per-key versioning to meta storage
to solve data write ordering issues in distributed environments.
```

## Adding a New Skill

To add a new skill:

1. Create a new directory under `.claude/skills/`
2. Add a `SKILL.md` file with YAML frontmatter and Markdown instructions
3. Commit to the repository

Basic `SKILL.md` format:

```markdown
---
name: my-skill
description: Describe what this skill does and when to use it.
---

# My Skill

Detailed instructions...
```

For more information, see the [Agent Skills specification](https://agentskills.io/).
