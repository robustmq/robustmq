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
use prometheus::{Registry, IntGauge};
use std::collections::HashMap;

lazy_static! {
    static ref SERVER_STATUS: IntGauge = IntGauge::new("server_status", "generic counter").unwrap();
}

const METRICS_SERVER_PRIFIX: &str = "robustmq_server";

pub fn register() {
    let mut labels = HashMap::new();
    labels.insert("name".to_string(), "loboxu".to_string());

    let registry = Registry::new_custom(Some(String::from(METRICS_SERVER_PRIFIX)), Some(labels)).unwrap();
    registry.register(Box::new(SERVER_STATUS.clone())).unwrap();
    SERVER_STATUS.set(0)
}

pub fn set_server_status_starting() {
    SERVER_STATUS.set(0)
}

pub fn set_server_status_running() {
    SERVER_STATUS.set(1)
}

pub fn set_server_status_stop() {
    SERVER_STATUS.set(2)
}
