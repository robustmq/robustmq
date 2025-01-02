# Github Contribution Guide
## 1. Create an ISSUE

There are two types of ISSUE scopes: requirements and minor fixes. Therefore, at the title level, they need to be distinguished by two prefixes: RBIP-* and MINOR.

**RBIP-***: It indicates the addition of features and functions. For example, RBIP-09, RBIP-10, and the subsequent serial numbers increase incrementally.
![image](../../images/doc-image9.png)

**MINOR**: It indicates the fixing or addition of some minor functions. The title can start with "MINOR:".
![image](../../images/doc-image10.png)

## 2. Create a Pull Request

If a PR is associated with an ISSUE, the following must be added to the content of the PR:

close #issue_number
"close" is a fixed prefix, and "#" is also a fixed prefix. "issue_number" represents the ISSUE number associated with this PR. For example:

![image](../../images/doc-image11.png)
#297 and #292 are the corresponding ISSUE numbers.

For example, if you need to submit a PR to solve ISSUE #297, the content of the PR should include "close #297". At this time, when this PR is merged, this ISSUE will be automatically closed. After the PR is merged, the effects of the PR and the ISSUE are as follows:

**PR**:
![image](../../images/doc-image12.png)

**ISSUE**:
![image](../../images/doc-image13.png)

## 3. Reasons for Failed PR Submission

### License Error

The License checker / license - header - check (pull_request) fails. Some files do not have a License added, and it needs to be added. It is recommended to execute the check command before each submission.

```
cargo install hawkeye

# Execute in the current project to detect which files do not have a License added
hawkeye check

# Automatically add a License to each code file
hawkeye format
```

### Incorrect Title Format

The failure of PR Title Checker / check (pull_request_target) is due to an incorrect PR title format.

```
Prefix: Title
The available prefix options are: feat|fix|test|refactor|chore|style|docs|perf|build|ci|revert

feat: New feature. For example: feat: Compatible with Rocksdb
fix: Bug fix
docs: Documentation changes
style: Changes in code style (do not affect code logic), used for submitting changes such as formatting, punctuation, and whitespace that do not affect code operation.
refactor: Code refactoring (code changes that are neither new feature additions nor bug fixes)
perf: Performance optimization
test: Addition or modification of tests
chore: Miscellaneous (changes in the build process or auxiliary tools)
build: Changes to the build system or external dependencies
ci: Changes to the continuous integration configuration, modification of configuration files and scripts.
revert: Revert
```
