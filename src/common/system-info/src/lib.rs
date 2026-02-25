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
pub use runtime::start_runtime_monitor;

use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_metrics::broker::{
    record_system_cpu_set, record_system_memory_set, record_system_process_cpu_set,
    record_system_process_memory_set,
};
use tokio::sync::broadcast;

pub async fn start_monitor(stop_send: broadcast::Sender<bool>, interval_ms: u64) {
    let interval_ms = interval_ms.max(100);
    let collect = async || -> ResultCommonError {
        // Values are stored as centipercent (×100); Grafana queries divide by 100.
        record_system_process_cpu_set((process_cpu_usage().await * 100.0).round() as i64);
        record_system_process_memory_set((process_memory_usage() * 100.0).round() as i64);
        record_system_cpu_set((system_cpu_usage().await * 100.0).round() as i64);
        record_system_memory_set((system_memory_usage() * 100.0).round() as i64);
        Ok(())
    };
    loop_select_ticket(collect, interval_ms, &stop_send).await;
}
