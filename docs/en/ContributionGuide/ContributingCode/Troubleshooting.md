# Troubleshooting

## `tokio-console` support and configuration

`tokio-console` is a tool that is commonly used to monitor and debug async rust applications built with the tokio runtime. Support for `tokio-console` can be enabled similar to how you would add a logging appender. The following entry in the `config.toml` file will add a `tokio-console` appender to the logging configuration:

```toml
[tokio_console] # This is the name of the appender, choose any name you like
kind = "TokioConsole" # Note that this is case-sensitive
bind = "127.0.0.1:5674" # Optional field to specify the server address for the tokio-console server
```

In order to collect task data from the tokio runtime, you also need to enable the `tokio_unstable` `cfg`. For example, you can run the placement center with the following command along with the above configuration to enable `tokio-console` support:

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo run --package cmd --bin placement-center
```

You can then use the following command to launch the `tokio-console` client to connect to the server listening on the custom address:

```bash
tokio-console http://127.0.1:5674
```
