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

use hdrhistogram::Histogram;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub struct BenchCounters {
    pub success: AtomicU64,
    pub failed: AtomicU64,
    pub timeout: AtomicU64,
    pub received: AtomicU64,
}

#[derive(Clone)]
pub struct SharedStats {
    pub counters: Arc<BenchCounters>,
    latency_us: Arc<Mutex<Histogram<u64>>>,
    errors: Arc<Mutex<BTreeMap<String, u64>>>,
}

impl SharedStats {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(BenchCounters::default()),
            latency_us: Arc::new(Mutex::new(
                Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).expect("histogram init failed"),
            )),
            errors: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn record_latency(&self, latency: Duration) {
        if let Ok(mut histogram) = self.latency_us.lock() {
            let _ = histogram.record(latency.as_micros().max(1) as u64);
        }
    }

    pub fn incr_success(&self) {
        self.counters.success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_failed(&self) {
        self.counters.failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_timeout(&self) {
        self.counters.timeout.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_received(&self) {
        self.counters.received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self, key: &str) {
        if let Ok(mut map) = self.errors.lock() {
            *map.entry(key.to_string()).or_insert(0) += 1;
        }
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        let success = self.counters.success.load(Ordering::Relaxed);
        let failed = self.counters.failed.load(Ordering::Relaxed);
        let timeout = self.counters.timeout.load(Ordering::Relaxed);
        let received = self.counters.received.load(Ordering::Relaxed);

        let (min, p50, p95, p99, max, mean) = if let Ok(h) = self.latency_us.lock() {
            if h.is_empty() {
                (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
            } else {
                (
                    h.min() as f64 / 1000.0,
                    h.value_at_quantile(0.50) as f64 / 1000.0,
                    h.value_at_quantile(0.95) as f64 / 1000.0,
                    h.value_at_quantile(0.99) as f64 / 1000.0,
                    h.max() as f64 / 1000.0,
                    h.mean() / 1000.0,
                )
            }
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

        let errors = self.errors.lock().map(|m| m.clone()).unwrap_or_default();

        StatsSnapshot {
            success,
            failed,
            timeout,
            received,
            latency_ms_min: min,
            latency_ms_p50: p50,
            latency_ms_p95: p95,
            latency_ms_p99: p99,
            latency_ms_max: max,
            latency_ms_avg: mean,
            errors,
        }
    }
}

impl Default for SharedStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsSnapshot {
    pub success: u64,
    pub failed: u64,
    pub timeout: u64,
    pub received: u64,
    pub latency_ms_min: f64,
    pub latency_ms_p50: f64,
    pub latency_ms_p95: f64,
    pub latency_ms_p99: f64,
    pub latency_ms_max: f64,
    pub latency_ms_avg: f64,
    pub errors: BTreeMap<String, u64>,
}
