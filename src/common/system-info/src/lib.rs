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

pub mod cpu;
pub mod fd;
pub mod memory;
pub mod runtime;

pub use cpu::{cpu_count, process_cpu_usage, system_cpu_usage};
pub use fd::{process_fd_count, system_fd_count};
pub use memory::{
    process_memory, process_memory_usage, system_memory_usage, total_memory, used_memory,
};
pub use runtime::start_tokio_runtime_info_collection;

use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_metrics::broker::{
    record_system_cpu_set, record_system_memory_set, record_system_process_cpu_set,
    record_system_process_memory_set,
};
use tokio::sync::broadcast;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_preserves_two_decimal_places() {
        assert_eq!(percentage(67.42), 67.42);
    }
}

pub async fn start_system_info_collection(stop_send: broadcast::Sender<bool>, interval_ms: u64) {
    let interval_ms = interval_ms.max(100);
    let collect = async || -> ResultCommonError {
        record_system_process_cpu_set(percentage(process_cpu_usage().await));
        record_system_process_memory_set(percentage(process_memory_usage()));
        record_system_cpu_set(percentage(system_cpu_usage().await));
        record_system_memory_set(percentage(system_memory_usage()));
        Ok(())
    };
    loop_select_ticket(collect, interval_ms, &stop_send).await;
}

fn percentage(value: f32) -> f64 {
    ((value as f64) * 100.0).round() / 100.0
}

#[cfg(test)]
mod collection_tests {
    use super::*;

    #[tokio::test]
    async fn system_info_collection_runs_and_stops_with_minimum_interval() {
        let (stop_send, _) = broadcast::channel(1);
        let collection = start_system_info_collection(stop_send.clone(), 0);
        tokio::pin!(collection);

        tokio::select! {
            biased;
            _ = &mut collection => panic!("system collector stopped before receiving a signal"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(450)) => {}
        }
        stop_send.send(true).unwrap();
        collection.await;

        let metrics = common_metrics::core::server::dump_metrics();
        for name in [
            "system_process_cpu_usage",
            "system_process_memory_usage",
            "system_cpu_usage",
            "system_memory_usage",
        ] {
            let line = metrics
                .lines()
                .find(|line| line.starts_with(&format!("{name}{{}} ")))
                .unwrap_or_else(|| panic!("missing sampled metric {name:?} in:\n{metrics}"));
            let value: f64 = line
                .split_whitespace()
                .last()
                .expect("metric sample should have a value")
                .parse()
                .expect("metric sample should be numeric");
            assert!(
                (0.0..=100.0).contains(&value),
                "{name} out of range: {line}"
            );
        }
    }
}
