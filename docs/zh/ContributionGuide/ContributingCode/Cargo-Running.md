# Cargo运行
1. Run standalone by placement-center
```
cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml
```
输出如下信息，表示 placement-center 启动成功:
![image](../../../images/Cargo-Running-1.png)

2. Run standalone by mqtt-server

```
cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
```

输出如下信息，表示 mqtt-server 启动成功：
![Cargo-Running-run-mqtt-server-2.png](../../../images/Cargo-Running-run-mqtt-server-2.png)

3. Run standalone by journal-server

```
cargo run --package cmd --bin journal-server -- --conf=config/journal-server.toml
```
输出如下信息，表示 journal-server 启动成功:
![Cargo-Running-run-journal-server-3.png](../../../images/Cargo-Running-run-journal-server-3.png)
