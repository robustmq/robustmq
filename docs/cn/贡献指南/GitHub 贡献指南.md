## 1、创建 ISSUE

ISSUE 范围两类，需求和小的 Fixed，所以在标题层面，需要通过：RBIP-\*和 MINOR 两个前缀来区分。

RBIP-\*： 是标识有特性和功能添加，比如 RBIP-09，RBIP-10，后面的序号是递增的。
![image](../../images/doc-image9.png)

MINOR：标识是修复或者增加一些小的功能。则可以 MINOR：开头，接标题。
![image](../../images/doc-image10.png)

## 2、创建 Pull Request

如果 PR 有关联的 ISSUE，必须在 PR 的内容中添加上：

close #issue_number
close 是固定的前缀， #也是固定的前缀，issue_number 表示这个 PR 关联的 ISSUE 编号。比如：

![image](../../images/doc-image11.png)
#297，#292 就是对应的 ISSUE 编号。

比如需要提交一个解决 ISSUE #297 的 PR，则 PR 内容需要包含

close #297
此时，当该 PR 被 MERGE 时，这个 ISSUE 会自动被关闭。PR 合并后，PR 和 ISSUE 效果如下：

PR：
![image](../../images/doc-image12.png)

ISSUE：
![image](../../images/doc-image13.png)

## 3、提交 PR 失败的原因

#### License 错误

License checker / license-header-check (pull_request) 失败。 有的文件没加 License, 需要加一下,最好每次提交前执行下检查命令.

```
cargo install hawkeye

# 当前项目下执行, 检测有哪些文件没有加 License
hawkeye check

# 自动没每个代码文件加上 License
hawkeye format
```

#### 标题格式错误

PR Title Checker / check (pull_request_target) 这个失败是 PR 的标题格式错误

```
前缀: 标题
前缀有这些选项：feat|fix|test|refactor|chore|style|docs|perf|build|ci|revert

feat: 新功能（feature）
例如：feat: Compatible with Rocksdb

fix: 修复 bug
docs: 文档变更
style: 代码风格变动（不影响代码逻辑）,用于提交仅格式化、标点符号、空白等不影响代码运行的变更。
refactor: 代码重构（既不是新增功能也不是修复bug的代码更改）
perf: 性能优化
test: 添加或修改测试
chore: 杂项（构建过程或辅助工具的变动）
build: 构建系统或外部依赖项的变更
ci: 持续集成配置的变更,配置文件和脚本的修改。
revert: 回滚
```
