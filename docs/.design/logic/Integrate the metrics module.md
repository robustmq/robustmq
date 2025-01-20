# 背景介绍
对于集成指标的背景就不赘述，这里给的背景是关于otel和prometheus的，
根据调研之后发现，可观测部分实现otel部分对于metrics部分还属于实验阶段，
由于otel是后续可观测性的标准协议，因此对于trace部分和log部分的集成，
依然会采用其rust的实现，但是对于metrics部分，业界通常使用的是prometheus，
并且在之前也有过open metrics的标准，最终发展后并入了prometheus当中，
因此在选择上，我们可以直接选择prometheus，总结一下相关选择理由：
1. prometheus的open metrics是业界对于metrics的一个事实标准
2. otel在rust这块儿还处于发展阶段，同时对于metrics指标最终也是集成到
   prometheus作为后端居多，不如一步到位
3. 在[prometheus3.0](https://github.com/prometheus/prometheus/releases/tag/v3.0.0)版本
上也能够对otel进行支持，即使后期有统一使用otel的rust库需求，依然可以
转换到otel库上

# 集成步骤
1. 选择prometheus库

由于prometheus前期并没有官方库进行支持，所以项目内部现在使用的是另一个
非官方库，虽然在功能上可能有些多，但是已经停止了维护[is this project still maintained](https://github.com/tikv/rust-prometheus/issues/530)
所以在这边实现上会采用[官方库](https://github.com/prometheus/client_rust)进行实现

2. 编写对应指标内容

对于指标编写这块儿，databend具有相应的metrics编写实现，所以最初编写实现
会参考databend的编写方式，集成到当前的实现当中

# 具体实现

1. 首先构建全局的mtrics注册对象，用于后续进行指标内容的注册
```rust
pub static REGISTRY: LazyLock<Mutex<Registry>> = LazyLock::new(|| Mutex::new(Registry::default()));

pub fn load_global_registry() -> MutexGuard<'static, Registry> {
    REGISTRY.lock().unwrap()
}
```
2. 构建指标存放集合
```rust
struct ConnectionJitterLabels {
    client_id: String,
}

struct ServerMetrics {
   connect_jitter_counter: Family<ConnectionJitterLabels, Counter>,
}

```
3. 设定初始化方法等

```rust
static SERVER_METRICS: LazyLock<ServerMetrics> = LazyLock::new(|| ServerMetrics::init());

impl ServerMetrics {


   fn init() -> Self {
      let metrics = Self {
         connect_jitter_counter: Family::default(),
      };

      let registry = load_global_registry();

      registry.register(
         "connect_jitter_counter",
         "current connect jitter times",
         metrics.connect_jitter_counter.clone(),
      );

      metrics
   }
}

pub fn incr_connect_jitter_counter(client_id: &str) {
   let labels = ConnectionJitterLabels {
      client_id: client_id.to_string(),
   };

   SERVER_METRICS.connect_jitter_counter.get_or_create(&labels).inc();
}

pub fn get_connect_jitter_counter(client_id: &str) -> i64 {
   let labels = ConnectionJitterLabels {
      client_id: client_id.to_string(),
   };

   SERVER_METRICS.connect_jitter_counter.get_or_create(&labels).get()
}
```
