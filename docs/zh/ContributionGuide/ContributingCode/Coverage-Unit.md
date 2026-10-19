# 本地单元测试覆盖率

本页面记录子问题 `#2079` 提供的可复用本地覆盖率命令。它针对现有单元测试范围
生成验证证据，不增加覆盖率门禁，不修改生产行为，也不会替代正确性测试流程。

## 前置条件

使用 `rust-toolchain.toml` 固定的 Rust 版本，并安装工具链验证任务中确认可用的工具：

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.6.21 --locked
cargo install cargo-nextest --version 0.9.84 --locked
```

工作区仍然需要现有的本地构建依赖。Linux 环境包括 CMake、Protobuf 和 OpenSSL
开发库等依赖。

## 本地运行

在仓库根目录执行：

```bash
make coverage-unit
```

等价的直接命令是：

```bash
./scripts/coverage-unit.sh
```

如果需要保留之前的结果，可以指定其他输出目录：

```bash
./scripts/coverage-unit.sh --output-dir /tmp/robustmq-coverage
```

脚本严格保持 `make test` 的测试范围：

- 选择工作区内的测试；
- 排除 `robustmq-test`；
- 排除 `grpc-clients`；
- 过滤掉 `storage-adapter` 的 `meta` 测试。

脚本使用 `cargo llvm-cov nextest --no-cfg-coverage`，关闭增量编译，并由 `cargo-llvm-cov` 为每个测试
进程使用包含 `%p-%10m` 的抗冲突 profile 文件名。插桩构建产物和 profile 文件都放在
指定输出目录中，不会与普通 `target/` 构建产物混合。每次运行都会使用干净的插桩
target，避免把旧 profile 混入报告。

`--no-cfg-coverage` 保留 LLVM 插桩，但不向依赖暴露 `cfg(coverage)` 条件。这在 stable
Rust 上是必要的，因为部分依赖会通过该条件开启仅 nightly 支持的特性。

## 输出约定

默认目录为 `target/coverage/unit`：

| 路径 | 用途 |
| --- | --- |
| `coverage.txt` | 人类可读的覆盖率报告 |
| `lcov.info` | 供后续 CI 或外部工具使用的 LCOV 报告 |
| `coverage.json` | LLVM 覆盖率导出的机器可读 JSON |
| `html/index.html` | 可浏览的 HTML 报告 |
| `coverage-summary.json` | 提交、范围、profile 数量、工具版本和报告路径 |
| `summary.txt` | 便于终端查看的简要摘要 |
| `run.log` | 测试和报告生成诊断日志 |

报告和插桩 profile 都是本地产物，不会提交到仓库。默认输出目录已被现有的
`target/` 忽略规则排除。

## 校验和失败行为

测试阶段失败时，命令会停止。测试通过后，脚本会检查覆盖率 target 中是否存在非空的
`.profraw` 文件；profile 缺失、为空或无效时，脚本会拒绝生成报告，避免产生虚假的
`0%` 结果。脚本还会验证 JSON 中包含已覆盖源码文件，并检查所有约定报告文件非空。

`coverage-summary.json` 会记录当前提交以及 Rust、Cargo、`cargo-llvm-cov`、
`cargo-nextest` 的准确版本。可以通过统计 LCOV 中的 `SF:` 记录和 JSON 中
`data[0].files` 的数量来比较两种报告的源码范围；HTML 报告可通过
`html/index.html` 查看已覆盖和未覆盖的源码文件。

该命令只测量进程内的单元测试和库测试。MQTT broker 进程、MQTT 集成场景、GitHub
Actions 产物、Codecov 以及覆盖率门禁属于后续子问题。
