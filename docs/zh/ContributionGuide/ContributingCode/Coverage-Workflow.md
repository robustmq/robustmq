# 单元测试覆盖率工作流

GitHub Actions 的 `Unit Coverage` 工作流为仓库现有的单元测试范围发布仅供
报告使用的覆盖率结果。它独立于普通单元测试、MQTT 集成测试和代码风格工作流，
在基线收集阶段不会设置覆盖率百分比门禁。

## 测试范围

工作流执行：

```text
make coverage-unit
```

范围与 `make test` 相同：排除 `robustmq-test` 和 `grpc-clients`，并过滤
`storage-adapter` 的 `meta` 测试。工作流会检出被测试的准确提交，并记录 Rust、
LLVM、覆盖率工具和 nextest 的版本。

缺少或无效的 profile、测试失败以及报告生成失败都会使工作流失败。基线收集
期间，覆盖率百分比降低不会单独导致工作流失败。

## 构件

每次运行都会尝试发布以下构件，包括失败的运行：

| 构件 | 内容 |
| --- | --- |
| `coverage-unit-reports` | 文本、LCOV、JSON、覆盖率元数据、工具链元数据和准确的测试命令 |
| `coverage-unit-html` | 可交互的 HTML 报告 |
| `coverage-unit-diagnostics` | 运行日志和工作流诊断信息 |

由于插桩后的 Cargo target 目录体积较大，且只是中间构建产物，因此不会上传。

## Actions 摘要

任务摘要明确标注为仅供报告使用，包含准确的单元测试范围、提交、运行结果、测试
数量、profile 数量和工具版本。如果文本报告包含 `TOTAL` 行，摘要会在 **Coverage
values** 部分展示该行。摘要还会列出全部可下载构件，便于维护者检查完整的 HTML、
LCOV、JSON 和诊断证据。

这个契约是后续 MQTT 冒烟测试和集成覆盖率工作流的单元覆盖率基线。后续工作流应
使用独立的构件名称和范围标签，不要合并不相关的覆盖率数值。
