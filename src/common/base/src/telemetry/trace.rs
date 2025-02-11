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

use crate::config::broker_mqtt::BrokerMqttConfig;
use opentelemetry::{global, propagation::Extractor, trace::noop::NoopTracerProvider};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
use std::{collections::HashMap, sync::OnceLock};

// Prepare to support three types
// 1. For performance, turn it off
// 2. For debugging, output to standard output
// 3. For release, output to otlp
#[derive(Debug)]
pub enum TraceExporterProvider {
    Noop(NoopTracerProvider),
    Stdout(NoopTracerProvider),
    Otlp(sdktrace::SdkTracerProvider),
}

static GLOBAL_PROVIDER: OnceLock<TraceExporterProvider> = OnceLock::new();

pub async fn init_tracer_provider(broker_config: &'static BrokerMqttConfig) {
    if !broker_config.telemetry.enable {
        global::set_tracer_provider(NoopTracerProvider::new());
        return;
    }
    match broker_config.telemetry.exporter_type.as_str() {
        "otlp" => {
            let exporter = SpanExporter::builder()
                .with_tonic()
                .with_endpoint(broker_config.telemetry.exporter_endpoint.as_str())
                .build()
                .unwrap();
            global::set_text_map_propagator(TraceContextPropagator::new());
            let provider = sdktrace::SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(Resource::builder().with_service_name("robustmq").build())
                .build();
            GLOBAL_PROVIDER
                .set(TraceExporterProvider::Otlp(provider.clone()))
                .unwrap();
            global::set_tracer_provider(provider.clone());
        }
        _ => {
            global::set_tracer_provider(NoopTracerProvider::new());
        }
    }
}

pub async fn stop_tracer_provider() {
    match GLOBAL_PROVIDER.get() {
        Some(provider) => {
            match provider {
                TraceExporterProvider::Otlp(provider) => {
                    provider.shutdown().unwrap();
                }
                TraceExporterProvider::Noop(_provider) => {
                    // Ignore
                }
                TraceExporterProvider::Stdout(_provider) => {
                    // Ignore
                }
            }
        }
        None => {}
    }
}

pub struct CustomContext {
    pub inner: HashMap<String, String>,
}

impl Extractor for CustomContext {
    /// Get a value for a key from the MetadataMap.  If the value can't be converted to &str, returns None
    fn get(&self, key: &str) -> Option<&str> {
        self.inner
            .get(key)
            .and_then(|metadata| Some(metadata.as_str()))
    }

    /// Collect all the keys from the MetadataMap.
    fn keys(&self) -> Vec<&str> {
        self.inner
            .keys()
            .map(|key| key.as_str())
            .collect::<Vec<_>>()
    }
}

impl CustomContext {
    pub fn new() -> Self {
        CustomContext {
            inner: HashMap::new(),
        }
    }
}

impl Default for CustomContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::broker_mqtt::broker_mqtt_conf;
    use crate::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use std::path::PathBuf;

    use super::init_tracer_provider;

    #[test]
    fn telemetry_config_test() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../../example/mqtt-cluster/mqtt-server/node-1.toml");
        init_broker_mqtt_conf_by_path(path.to_str().unwrap());
        let telemetry_config = &broker_mqtt_conf().telemetry;
        assert_eq!(telemetry_config.enable, true);
        assert_eq!(
            telemetry_config.exporter_endpoint,
            "grpc://127.0.0.1:4317".to_string()
        );
        assert_eq!(telemetry_config.exporter_type, "otlp".to_string());
    }

    #[tokio::test]
    async fn telemetry_test_init() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../../example/mqtt-cluster/mqtt-server/node-1.toml");
        init_broker_mqtt_conf_by_path(path.to_str().unwrap());
        init_tracer_provider(&broker_mqtt_conf()).await
    }
}
