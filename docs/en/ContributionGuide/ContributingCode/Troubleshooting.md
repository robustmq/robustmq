# Troubleshooting

## `tokio-console` support and configuration

`tokio-console` is a tool that is commonly used to monitor and debug async rust applications built with the tokio runtime. Support for `tokio-console` can be enabled by configuring the `tokio_console` config in the `RUSTFLAGS` environment variable. For example, the following command will launch the placement center with `tokio-console` support:

```bash
RUSTFLAGS="--cfg tokio_unstable --cfg tokio_console"  cargo run --package cmd --bin placement-center
```

### Configuring the socket address with environment variables

`tokio-console` will use the [default IP](https://docs.rs/console-subscriber/latest/console_subscriber/struct.Server.html#associatedconstant.DEFAULT_IP) and [default port](https://docs.rs/console-subscriber/latest/console_subscriber/struct.Server.html#associatedconstant.DEFAULT_PORT) to listen for RPC connections. If you want to configure the socket address, you can use the the environment variable `TOKIO_CONSOLE_BIND` to specify the address. For example, the following command will launch the placement center with `tokio-console` support and bind to `127.0.0.1:5674`:

```bash
export TOKIO_CONSOLE_BIND=127.0.0.1:5674 && RUSTFLAGS="--cfg tokio_unstable --cfg tokio_console"  cargo run --package cmd --bin placement-center
```

You can then use the following command to launch the `tokio-console` client to connect to the server listening on the custom address:

```bash
tokio-console http://127.0.0.1:5674
```

### Configuring `tokio-console` with config files

Support for configuring `tokio-console` with a config file is still under development. This section will be updated once the feature is implemented.
