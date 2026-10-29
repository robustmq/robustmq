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

pub mod broker;
pub mod common;
pub mod core;
pub mod grpc;
pub mod http;
pub mod meta;
pub mod mqtt;
pub mod network;
pub mod rocksdb;
pub mod storage_engine;

#[cfg(test)]
pub(crate) static METRIC_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Pre-register all static-label gauge metrics to 0 so that they appear in
/// the Prometheus `/metrics` output immediately on startup, even before any
/// real traffic has occurred.  Call this once during broker initialisation.
pub fn init_metrics() {
    mqtt::init();
    broker::init();
    meta::raft::init();
    network::init();
    storage_engine::init();
}

#[cfg(test)]
mod tests {
    use super::init_metrics;

    #[test]
    fn initializes_all_static_metric_modules() {
        let _guard = super::METRIC_TEST_LOCK.lock().unwrap();
        init_metrics();
        assert!(crate::core::server::dump_metrics().contains("mqtt_connections_count"));
        assert!(crate::core::server::dump_metrics().contains("tokio_runtime_queue_depth"));
        assert!(crate::core::server::dump_metrics().contains("storage_engine_ops"));
    }
}
