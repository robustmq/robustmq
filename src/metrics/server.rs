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

use lazy_static::lazy_static;
use prometheus::{Encoder, IntGauge, Registry};
use std::collections::HashMap;
use std::fmt;

lazy_static! {
    static ref SERVER_STATUS: IntGauge = IntGauge::new("server_status", "generic counter").unwrap();
}

const METRICS_SERVER_PRIFIX: &str = "robustmq_server";

pub struct ServerMetrics {
    pub registry: Registry,
}

impl fmt::Debug for ServerMetrics{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
         .field("x", &String::from("value"))
         .finish()
    }
}

impl ServerMetrics {
    pub fn new() -> Self {
        ServerMetrics {
            registry: ServerMetrics::build_registry(),
        }
    }

    fn build_registry() -> Registry {
        let mut labels = HashMap::new();
        labels.insert("name".to_string(), "loboxu".to_string());
        Registry::new_custom(Some(String::from(METRICS_SERVER_PRIFIX)), Some(labels)).unwrap()
    }

    pub fn init(&self) {
        self.registry
            .register(Box::new(SERVER_STATUS.clone()))
            .unwrap();
    }

    pub fn _set_server_status_starting(&self) {
        SERVER_STATUS.set(0);
    }

    pub fn set_server_status_running(&self) {
        SERVER_STATUS.set(1);
    }

    pub fn set_server_status_stop(&self) {
        SERVER_STATUS.set(2);
    }

    pub fn gather(&self) -> String {
        let mut buf = Vec::<u8>::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&self.registry.gather(), &mut buf).unwrap();
        return String::from_utf8(buf.clone()).unwrap();
    }
}
