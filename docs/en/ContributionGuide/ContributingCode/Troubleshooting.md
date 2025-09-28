# Troubleshooting

## `tokio-console` support and configuration

`tokio-console` is a tool that is commonly used to monitor and debug async rust applications built with the tokio runtime. Support for `tokio-console` can be enabled similar to how you would add a logging appender. The following entry in the `config/server-tracing.toml` file will add a `tokio-console` appender to the logging configuration:

```toml
[tokio_console] # This is the name of the appender, choose any name you like
kind = "tokio_console" # Note that this is case-sensitive
bind = "127.0.0.1:5674" # specify the server address for the tokio-console server(default: 127.0.0.1:6669)
```

In order to collect task data from the tokio runtime, you also need to enable the `tokio_unstable` `cfg`. For example, you can run the meta service with the following command along with the above configuration to enable `tokio-console` support:

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo run --package cmd --bin meta-service
```

You can then use the following command to launch the `tokio-console` client to connect to the server listening on the custom address:

```bash
tokio-console http://127.0.0.1:5674
```

You can also configure `grpc_web` to enable web client request support. Please note that the current console default port is set to 6669. For security reasons, some browsers may restrict this port.

```toml
[tokio_console]
grpc_web = true # Set whether to enable the grpc-web support (default: false)
```
