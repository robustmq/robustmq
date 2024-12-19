1. Run standalone by placement-center
```
cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml
```
输出如下信息，表示 placement-center 启动成功:
![image](../../images/doc-image6.png)

2. Run standalone by mqtt-server

```
cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
```

输出如下信息，标识 mqtt-server 启动成功：
![image](../../images/doc-image7.png)
