# 故障排查

## `tokio-console` 的支持和配置

`tokio-console` 常被用来调试使用`tokio`运行时构建的异步 Rust 应用程序。可以通过在 `RUSTFLAGS` 环境变量中配置 `tokio_console` 来启用对 `tokio-console` 的支持。例如，以下命令将启动带有 `tokio-console` 支持的 placement center：

```bash
RUSTFLAGS="--cfg tokio_unstable --cfg tokio_console"  cargo run --package cmd --bin placement-center
```

### 使用环境变量配置socket地址

`tokio-console`默认使用[默认IP](https://docs.rs/console-subscriber/latest/console_subscriber/struct.Server.html#associatedconstant.DEFAULT_IP)和[默认端口](https://docs.rs/console-subscriber/latest/console_subscriber/struct.Server.html#associatedconstant.DEFAULT_PORT)来监听RPC连接。如果你想配置socket地址，可以使用环境变量`TOKIO_CONSOLE_BIND`来指定地址。例如，以下命令将启动带有 `tokio-console` 支持的 placement center，并绑定到地址 `127.0.0.1:5674`：

```bash
export TOKIO_CONSOLE_BIND=127.0.0.1:5674 && RUSTFLAGS="--cfg tokio_unstable --cfg tokio_console"  cargo run --package cmd --bin placement-center
```

以下命令将启动 `tokio-console` 客户端以连接到监听在自定义地址上的服务器：

```bash
tokio-console http://127.0.0.1:5674
```

### 通过配置文件配置 `tokio-console`

对通过配置文件配置 `tokio-console` 的支持仍在开发中。此部分内容将于功能实现后更新。
