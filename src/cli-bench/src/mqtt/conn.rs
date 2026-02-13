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

use crate::error::BenchMarkError;
use crate::mqtt::common::{build_client, wait_connack};
use crate::mqtt::report::{print_realtime_line, BenchReport, BenchReportInput, ThroughputSample};
use crate::mqtt::stats::SharedStats;
use crate::mqtt::{ConnBenchArgs, ConnMode, OutputFormat};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

pub async fn run_conn_bench(args: ConnBenchArgs) -> Result<(), BenchMarkError> {
    let bench_start = Instant::now();
    let mode = args.mode.clone();
    let hold_duration = Duration::from_secs(args.hold_secs);
    let stats = SharedStats::new();
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(args.common.count);

    for i in 0..args.common.count {
        if args.common.interval_ms > 0 {
            tokio::time::sleep(Duration::from_millis(args.common.interval_ms)).await;
        }
        let client_id = format!("robust-bench-conn-{i}");
        let host = args.common.host.clone();
        let port = args.common.port;
        let username = args.common.username.clone();
        let password = args.common.password.clone();
        let local_stats = stats.clone();
        let mode = mode.clone();
        let hold_duration_each = hold_duration;

        handles.push(tokio::spawn(async move {
            let start = Instant::now();
            let (client, mut event_loop) =
                build_client(&client_id, &host, port, &username, &password);
            match wait_connack(&mut event_loop, 10_000).await {
                Ok(_) => {
                    local_stats.incr_success();
                    local_stats.record_latency(start.elapsed());
                }
                Err(e) => {
                    local_stats.incr_failed();
                    local_stats.record_error(&format!("connect:{e}"));
                    return;
                }
            }
            if matches!(mode, ConnMode::Hold) {
                let hold_deadline = tokio::time::Instant::now() + hold_duration_each;
                while tokio::time::Instant::now() < hold_deadline {
                    if tokio::time::timeout(Duration::from_millis(100), event_loop.poll())
                        .await
                        .is_err()
                    {
                        continue;
                    }
                }
            }
            let _ = client.disconnect().await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
    let snapshot = stats.snapshot();
    let elapsed = bench_start.elapsed();
    let total_ops = snapshot.success;
    print_realtime_line("conn", elapsed, total_ops, total_ops, &snapshot);
    let series = vec![ThroughputSample {
        second: elapsed.as_secs(),
        ops_per_sec: total_ops,
        total_ops,
        success: snapshot.success,
        failed: snapshot.failed,
        timeout: snapshot.timeout,
        received: snapshot.received,
    }];
    let mut extras = BTreeMap::new();
    extras.insert("mode".to_string(), format!("tcp-conn:{:?}", args.mode));
    extras.insert(
        "interval_ms".to_string(),
        args.common.interval_ms.to_string(),
    );
    extras.insert("hold_secs".to_string(), args.hold_secs.to_string());
    extras.insert("qos".to_string(), args.common.qos.to_string());

    let report = BenchReport::from_input(
        BenchReportInput {
            name: "mqtt-conn".to_string(),
            host: args.common.host,
            port: args.common.port,
            duration_secs: elapsed.as_secs(),
            clients: args.common.count,
            op_label: "conn_ack".to_string(),
            total_ops,
            extras,
            series,
        },
        snapshot,
    );
    match args.common.output {
        OutputFormat::Table => report.print_table(),
        OutputFormat::Json => report.print_json(),
    }
    Ok(())
}
