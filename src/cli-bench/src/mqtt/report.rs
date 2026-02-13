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

use crate::mqtt::stats::StatsSnapshot;
use prettytable::{row, Table};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct ThroughputSample {
    pub second: u64,
    pub ops_per_sec: u64,
    pub total_ops: u64,
    pub success: u64,
    pub failed: u64,
    pub timeout: u64,
    pub received: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchReport {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub duration_secs: u64,
    pub clients: usize,
    pub op_label: String,
    pub total_ops: u64,
    pub avg_ops_per_sec: f64,
    pub peak_ops_per_sec: u64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub extras: BTreeMap<String, String>,
    pub series: Vec<ThroughputSample>,
    pub snapshot: StatsSnapshot,
}

#[derive(Debug, Clone)]
pub struct BenchReportInput {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub duration_secs: u64,
    pub clients: usize,
    pub op_label: String,
    pub total_ops: u64,
    pub extras: BTreeMap<String, String>,
    pub series: Vec<ThroughputSample>,
}

impl BenchReport {
    pub fn from_input(input: BenchReportInput, snapshot: StatsSnapshot) -> Self {
        let avg_ops_per_sec = if input.duration_secs == 0 {
            0.0
        } else {
            input.total_ops as f64 / input.duration_secs as f64
        };
        let peak_ops_per_sec = input
            .series
            .iter()
            .map(|s| s.ops_per_sec)
            .max()
            .unwrap_or(0);
        let total_attempts = snapshot.success + snapshot.failed + snapshot.timeout;
        let (success_rate, error_rate, timeout_rate) = if total_attempts == 0 {
            (0.0, 0.0, 0.0)
        } else {
            (
                snapshot.success as f64 * 100.0 / total_attempts as f64,
                snapshot.failed as f64 * 100.0 / total_attempts as f64,
                snapshot.timeout as f64 * 100.0 / total_attempts as f64,
            )
        };

        Self {
            name: input.name,
            host: input.host,
            port: input.port,
            duration_secs: input.duration_secs,
            clients: input.clients,
            op_label: input.op_label,
            total_ops: input.total_ops,
            avg_ops_per_sec,
            peak_ops_per_sec,
            success_rate,
            error_rate,
            timeout_rate,
            extras: input.extras,
            series: input.series,
            snapshot,
        }
    }

    pub fn print_table(&self) {
        println!("=== Benchmark Summary ===");
        let mut table = Table::new();
        table.set_titles(row!["metric", "value"]);
        table.add_row(row!["name", self.name.as_str()]);
        table.add_row(row!["target", format!("{}:{}", self.host, self.port)]);
        table.add_row(row!["duration(s)", self.duration_secs]);
        table.add_row(row!["clients", self.clients]);
        table.add_row(row!["op_label", self.op_label.as_str()]);
        table.add_row(row!["total_ops", self.total_ops]);
        table.add_row(row![
            "avg_ops_per_sec",
            format!("{:.2}", self.avg_ops_per_sec)
        ]);
        table.add_row(row!["peak_ops_per_sec", self.peak_ops_per_sec]);
        table.add_row(row!["success", self.snapshot.success]);
        table.add_row(row!["failed", self.snapshot.failed]);
        table.add_row(row!["timeout", self.snapshot.timeout]);
        table.add_row(row!["received", self.snapshot.received]);
        table.add_row(row!["success_rate(%)", format!("{:.4}", self.success_rate)]);
        table.add_row(row!["error_rate(%)", format!("{:.4}", self.error_rate)]);
        table.add_row(row!["timeout_rate(%)", format!("{:.4}", self.timeout_rate)]);
        table.add_row(row![
            "latency_avg(ms)",
            format!("{:.3}", self.snapshot.latency_ms_avg)
        ]);
        table.add_row(row![
            "latency_p50(ms)",
            format!("{:.3}", self.snapshot.latency_ms_p50)
        ]);
        table.add_row(row![
            "latency_p95(ms)",
            format!("{:.3}", self.snapshot.latency_ms_p95)
        ]);
        table.add_row(row![
            "latency_p99(ms)",
            format!("{:.3}", self.snapshot.latency_ms_p99)
        ]);
        table.add_row(row![
            "latency_min(ms)",
            format!("{:.3}", self.snapshot.latency_ms_min)
        ]);
        table.add_row(row![
            "latency_max(ms)",
            format!("{:.3}", self.snapshot.latency_ms_max)
        ]);
        table.printstd();

        if !self.extras.is_empty() {
            println!("=== Scenario Parameters ===");
            let mut ext = Table::new();
            ext.set_titles(row!["param", "value"]);
            for (k, v) in &self.extras {
                ext.add_row(row![k, v]);
            }
            ext.printstd();
        }

        if !self.series.is_empty() {
            println!("=== Throughput Timeline (per second) ===");
            let mut timeline = Table::new();
            timeline.set_titles(row![
                "sec",
                "ops/s",
                "total_ops",
                "success",
                "failed",
                "timeout",
                "received"
            ]);
            for sample in &self.series {
                timeline.add_row(row![
                    sample.second,
                    sample.ops_per_sec,
                    sample.total_ops,
                    sample.success,
                    sample.failed,
                    sample.timeout,
                    sample.received
                ]);
            }
            timeline.printstd();
        }

        if !self.snapshot.errors.is_empty() {
            println!("=== Error Distribution ===");
            let mut err = Table::new();
            err.set_titles(row!["error_type", "count"]);
            for (k, v) in &self.snapshot.errors {
                err.add_row(row![k, v]);
            }
            err.printstd();
        }
    }

    pub fn print_json(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(raw) => println!("{raw}"),
            Err(e) => eprintln!("failed to print json report: {e}"),
        }
    }
}

pub fn print_realtime_line(
    stage: &str,
    elapsed: Duration,
    delta_ops: u64,
    total_ops: u64,
    snapshot: &StatsSnapshot,
) {
    println!(
        "[{}] sec={} ops/s={} total_ops={} success={} failed={} timeout={} recv={} p95={:.3}ms p99={:.3}ms",
        stage,
        elapsed.as_secs(),
        delta_ops,
        total_ops,
        snapshot.success,
        snapshot.failed,
        snapshot.timeout,
        snapshot.received,
        snapshot.latency_ms_p95,
        snapshot.latency_ms_p99
    );
}
