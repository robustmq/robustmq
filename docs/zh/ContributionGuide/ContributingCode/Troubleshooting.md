# 故障排查

## `tokio-console` 的支持和配置

`tokio-console` 常被用来调试使用`tokio`运行时构建的异步 Rust 应用程序。要启用对 `tokio-console` 的支持，可以像添加日志appender一样进行配置。以下 `config.toml` 文件中的条目将向日志配置中添加一个 `tokio-console` appender：

```toml
[tokio_console] # 这是 appender 的名称，可以选择任何名称
kind = "TokioConsole" # 注意这是区分大小写的
bind = "127.0.0.1:5674“ # 非必须字段，用于指定 tokio-console 服务器的地址
```

要从 tokio 运行时收集任务数据，还需要启用 `tokio_unstable` `cfg`。例如，可以使用以下命令结合上述配置启用 `tokio-console` 支持并运行 placement center ：

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo run --package cmd --bin placement-center
```

然后，可以使用以下命令启动 `tokio-console` 客户端，连接到监听自定义地址的服务器：

```bash
tokio-console http://127.0.1:5674
```
