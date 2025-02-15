// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use common_base::tools::unique_id;

use paho_mqtt::{Message, MessageBuilder, Properties, PropertyCode};

use opentelemetry::{global, propagation::Injector};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace};
// use opentelemetry_stdout::SpanExporter;

use opentelemetry::{
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};

use crate::mqtt_protocol::common::{broker_addr, connect_server34, connect_server5, distinct_conn};

async fn publish34_qos(num: i32, qos: i32) {
    let mqtt_version = 3;
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server34(mqtt_version, &client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();
    for i in 0..num {
        let msg = Message::new(topic.clone(), format!("mqtt {i} message"), qos);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    distinct_conn(cli);

    let mqtt_version = 4;
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server34(mqtt_version, &client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();
    for i in 0..num {
        let msg = Message::new(topic.clone(), format!("mqtt {i} message"), qos);
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    distinct_conn(cli);
}

async fn publish5_qos(num: i32, qos: i32, retained: bool) {
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server5(&client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();

    let mut props = Properties::new();
    props
        .push_u32(PropertyCode::MessageExpiryInterval, 50)
        .unwrap();
    for i in 0..num {
        let payload = format!("mqtt {i} message");
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(payload)
            .topic(topic.clone())
            .qos(qos)
            .retained(retained)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    distinct_conn(cli);
}

async fn publish5_qos_trace(num: i32, qos: i32, retained: bool) {
    let client_id = unique_id();
    let addr = broker_addr();
    let cli = connect_server5(&client_id, &addr, false, false);
    let topic = "/tests/t1".to_string();

    //  A program may require many targets
    let tracer = global::tracer("mqtt/client");
    let span = tracer
        .span_builder("cli/publish")
        .with_kind(SpanKind::Client)
        .with_attributes([KeyValue::new("component", "mqtt")])
        .start(&tracer); // start span

    // Tracing of distributed systems requires cross-system context
    let cx = Context::current_with_span(span);

    let mut new_map = MqttContext::default();
    let mut props = Properties::default();

    global::get_text_map_propagator(|propagator| propagator.inject_context(&cx, &mut new_map));

    new_map.inner.iter().for_each(|(key, value)| {
        println!("key: {}, value: {}", key, value);
        props
            .push_string_pair(PropertyCode::UserProperty, key, value)
            .unwrap();
    });

    props
        .push_u32(PropertyCode::MessageExpiryInterval, 50)
        .unwrap();
    for i in 0..num {
        let payload = format!("mqtt {i} message");
        let msg = MessageBuilder::new()
            .properties(props.clone())
            .payload(payload)
            .topic(topic.clone())
            .qos(qos)
            .retained(retained)
            .finalize();
        match cli.publish(msg) {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    cx.span().add_event(
        "Got response!",
        vec![KeyValue::new("status", 200.to_string())],
    );
    // If the scope is not left, you must manually end() to ensure the closure of the span.
    cx.span().end();
    distinct_conn(cli);
}

#[derive(Debug)]
pub struct MqttContext {
    pub inner: HashMap<String, String>,
}

impl MqttContext {
    pub fn new() -> Self {
        MqttContext {
            inner: HashMap::new(),
        }
    }
}
impl Injector for MqttContext {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        println!("Injecting: {} -> {}", key, value);
        self.inner.insert(key.to_string(), value);
    }
}

impl Default for MqttContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time;

    use opentelemetry_otlp::SpanExporter;
    use paho_mqtt::{QOS_0, QOS_1, QOS_2};

    use crate::mqtt_protocol::pub_qos_test::{publish34_qos, publish5_qos, publish5_qos_trace};

    #[tokio::test]
    async fn client34_publish_test() {
        let num = 10;
        publish34_qos(num, QOS_0).await;
        publish34_qos(num, QOS_1).await;
        publish34_qos(num, QOS_2).await;
    }

    #[tokio::test]
    async fn client5_publish_test() {
        let num = 1;
        publish5_qos(num, QOS_0, false).await;
        publish5_qos(num, QOS_1, false).await;
        publish5_qos(num, QOS_2, false).await;

        publish5_qos(num, QOS_0, true).await;
        publish5_qos(num, QOS_1, true).await;
        publish5_qos(num, QOS_2, true).await;
    }

    #[tokio::test]
    #[ignore]
    async fn client5_publish_trace_test() {
        let num = 1;
        let exporter = SpanExporter::builder().with_tonic().build().unwrap();
        global::set_text_map_propagator(TraceContextPropagator::new());
        let provider = sdktrace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(Resource::builder().with_service_name("mqtt-cli").build())
            .build();
        global::set_tracer_provider(provider.clone());
        publish5_qos_trace(num, QOS_1, false).await;
        tokio::time::sleep(time::Duration::from_millis(10000)).await;
        match provider.shutdown() {
            Ok(_) => {
                println!("shutdown success");
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
