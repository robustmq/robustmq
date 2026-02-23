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

pub use cpu::{cpu_count, process_cpu_usage, system_cpu_usage};
pub use fd::{process_fd_count, system_fd_count};
pub use memory::{
    process_memory, process_memory_usage, system_memory_usage, total_memory, used_memory,
};

use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use common_metrics::broker::{
    record_system_cpu_set, record_system_memory_set, record_system_process_cpu_set,
    record_system_process_memory_set,
};
use tokio::sync::broadcast;
const MONITOR_INTERVAL_MS: u64 = 15_000;

pub async fn start_monitor(stop_send: broadcast::Sender<bool>) {
    let collect = async || -> ResultCommonError {
        // Multiply by 100 before storing to preserve two decimal places of precision
        // (e.g. 0.63% is stored as 63). Grafana queries must divide by 100.
        let proc_cpu = process_cpu_usage().await;
        record_system_process_cpu_set((proc_cpu * 100.0).round() as i64);

        let proc_mem = process_memory_usage();
        record_system_process_memory_set((proc_mem * 100.0).round() as i64);

        let sys_cpu = system_cpu_usage().await;
        record_system_cpu_set((sys_cpu * 100.0).round() as i64);

        let sys_mem = system_memory_usage();
        record_system_memory_set((sys_mem * 100.0).round() as i64);

        Ok(())
    };

    loop_select_ticket(collect, MONITOR_INTERVAL_MS, &stop_send).await;
}
