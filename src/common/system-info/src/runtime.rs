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

use common_metrics::broker::{
    record_runtime_alive_tasks_set, record_runtime_busy_ratio_set, record_runtime_queue_depth_set,
};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

struct RuntimeSnapshot {
    name: String,
    handle: tokio::runtime::Handle,
    prev_busy: Vec<Duration>,
    prev_time: Instant,
}

impl RuntimeSnapshot {
    fn new(name: String, handle: tokio::runtime::Handle) -> Self {
        let m = handle.metrics();
        let prev_busy = (0..m.num_workers())
            .map(|i| m.worker_total_busy_duration(i))
            .collect();
        Self {
            name,
            handle,
            prev_busy,
            prev_time: Instant::now(),
        }
    }

    fn sample(&mut self) {
        let m = self.handle.metrics();
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.prev_time);
        let n = m.num_workers();

        let busy_delta: Duration = (0..n)
            .map(|i| {
                m.worker_total_busy_duration(i)
                    .saturating_sub(self.prev_busy.get(i).copied().unwrap_or_default())
            })
            .sum();

        let busy_ratio = if n > 0 && !elapsed.is_zero() {
            busy_delta.as_secs_f64() / elapsed.saturating_mul(n as u32).as_secs_f64()
        } else {
            0.0
        };

        let alive = m.num_alive_tasks();
        let queue = m.global_queue_depth();

        record_runtime_busy_ratio_set(&self.name, (busy_ratio * 100.0).round() as i64);
        record_runtime_queue_depth_set(&self.name, queue as i64);
        record_runtime_alive_tasks_set(&self.name, alive as i64);

        self.prev_busy = (0..n).map(|i| m.worker_total_busy_duration(i)).collect();
        self.prev_time = now;
    }
}

pub async fn start_runtime_monitor(
    handles: Vec<(String, tokio::runtime::Handle)>,
    stop_send: broadcast::Sender<bool>,
    interval_ms: u64,
) {
    let interval_ms = interval_ms.max(100);
    let mut snapshots: Vec<RuntimeSnapshot> = handles
        .into_iter()
        .map(|(n, h)| RuntimeSnapshot::new(n, h))
        .collect();

    let mut stop_recv = stop_send.subscribe();
    let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                for snap in &mut snapshots {
                    snap.sample();
                }
            }
            val = stop_recv.recv() => {
                if matches!(val, Ok(true) | Err(_)) { break; }
            }
        }
    }
}
