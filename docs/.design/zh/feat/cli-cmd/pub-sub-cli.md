1. publish

    使用clap解析命令行参数

    ```rust
    #[derive(Clone, Debug, PartialEq)]
    pub struct PublishArgsRequest {
        pub topic: String,
        pub qos: i32,
        pub retained: bool,
        pub username: String,
        pub password: String,
    }

    ```
    实现发布命令

    ```bash
    cargo run --package cmd --bin cli-command -- mqtt --server=127.0.0.1:1883   publish --username=admin --password=pwd123 --topic=test/topic1 --qos=0
    ```
    启动异步运行时   `loop` + `select` 读取用户 `stdin` 键盘输出数据，循环发送到 broker，并且监听 `CTR+C` 退出程序主动关闭MQTT连接




2. 定义消息发布和订阅的逻辑

    ```bash
    cargo run --package cmd --bin cli-command -- mqtt --server=127.0.0.1:1883   subscribe --username=admin --password=pwd123 --topic=test/topic1 --qos=0
    ```
    核心逻辑同步循环接收message，异步监听`CTR+C` 退出程序，关闭MQTT连接，并且显示退出信息
    ```rust
    while let Some(msg) = rx.iter().next() {
    }
    ```
