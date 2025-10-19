# PR 提交流程示例

本示例详细说明了如何提交一个 Pull Request（以下简称 PR）以修复项目中的 Bug。

## 一、准备工作

请确保你已经完成[开发环境的配置](ContributingCode/Build-Develop-Env.md)。

## 二、完整 PR 流程

### 1. Fork 项目

- 前往项目仓库：[robustmq](https://github.com/robustmq/robustmq)
- 点击页面右上角的 **Star** 按钮，然后点击 **Fork** 按钮。
- 复制 Fork 后的仓库地址，例如：`https://github.com/你的用户名/robustmq.git`

### 2. 克隆仓库

```shell
mkdir work
cd work
git clone https://github.com/你的用户名/robustmq.git
```

### 3. 创建新分支

```shell
cd robustmq
git checkout -b pr-example
```

### 4. 修改代码

示例代码修改如下：

```rust
#[tokio::main]
async fn main() {
    // 添加新功能或修复Bug的代码
    println!("Hello, robustmq!");
    let args = ArgsParams::parse();
    init_meta_service_conf_by_path(&args.conf);
    init_meta_service_log();
    let (stop_send, _) = broadcast::channel(2);
    let mut pc = MetaService::new();
    pc.start(stop_send).await;
}
```

### 5. 提交更改

如果配置了 `pre-commit`，提交前会自动执行代码检查和单元测试。

```shell
git add .
git commit -m "fix: 修复示例Bug"
```

### 6. 集成测试

务必进行以下代码质量检查和测试：

| 测试类型                  | 命令                     |
|---------------------------|-------------------------|
| 代码质量检查              | `make codecheck`        |
| 单元测试                  | `make test`             |
| 所有测试（单元+集成）     | `make test-all`         |
| 仅 MQTT 集成测试          | `make mqtt-ig-test`     |

```shell
# 运行代码质量检查（格式化、clippy、许可证）
make codecheck

# 运行所有测试
make test-all

# 或分别运行
make test
make mqtt-ig-test
```

以上测试通过后，提交分支：

```shell
git push origin pr-example
```

### 7. 处理主仓库更新（可选）

若主仓库有更新，需合并到你的分支后再提交：

```shell
# 添加原始项目仓库（仅首次添加）
git remote add upstream https://github.com/robustmq/robustmq.git
# 拉取原始仓库最新代码
git fetch upstream
# 确保当前位于你的分支
git checkout pr-example
# 合并主分支最新代码
git merge upstream/main
# 再次执行测试
make test-all
# 提交合并后的代码
git commit -m "Merge upstream main into pr-example"
git push origin pr-example
```

### 8. 创建 PR

前往你的 Fork 仓库页面，点击 **New Pull Request** 按钮，填写标题和描述：

```markdown
标题：fix: 修复示例Bug
描述：修复了某某功能的具体问题，确保xxx功能正常工作。

close #ISSUE_NUMBER（如果关联了 ISSUE）
```

点击 **Create Pull Request** 完成创建。

### 9. 等待审核和合并

GitHub CI 将自动检查代码，等待项目维护者审核。审核通过后你的 PR 将被合并。

---
