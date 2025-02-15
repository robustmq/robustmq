1. 增加配置telemetry相关内容

    ```toml

    [telemetry]
    enable = true # 是否开启
    exporter_type = "otlp" # otlp 类型
    exporter_endpoint = "grpc://127.0.0.1:4317" # 导出的数据，这里是用 jaegertracing/all-in-one:latest

    ```

    在 telemetry 中，如果需要跨系统进行追踪需要在不同系统之间传递  Context  。Context 中最重要的数据是

    ```json

    {
    "traceparent": "00-7ba70cddfcc3f1135b659866752a5556-ca61e958b8f40d0c-01",
    "tracestate": "",
    }

    ```
    traceparent 确保了，系统追踪的连贯性。

2. 创建了 CustomContext 接收传递的 parent_id   ，用于存储分布式其他程序传递的 Context

3. 创建 `src/common/base/src/telemetry/trace.rs` 文件,实现 `init_tracer_provider` 和 `stop_tracer_provider` 方法

4. 在 start 方法中调用 `init_tracer_provider`。在 stop 方法中调用 `stop_tracer_provider`

5. 在需要 trace 的地方，调用

```
                let tracer = global::tracer("robustmq/publish");
                let mut span = tracer
                    .span_builder("command/publish")
                    .with_kind(SpanKind::Server)
                    .start_with_context(&tracer, &parent_cx); // 简单的可以理解为这个 span 的计时开始


                // span add_event 计时结束
                span.add_event(
                    format!("connection_id: {:?}", tcp_connection.connection_id),
                    vec![],
                );

```
