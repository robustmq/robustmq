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
use crate::mqtt::report::{
    print_conn_progress_line, print_realtime_line, BenchReport, BenchReportInput, ThroughputSample,
};
use crate::mqtt::stats::SharedStats;
use crate::mqtt::{ConnBenchArgs, ConnMode, OutputFormat};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub async fn run_conn_bench(args: ConnBenchArgs) -> Result<(), BenchMarkError> {
    if args.concurrency == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "concurrency must be greater than 0".to_string(),
        ));
    }

    let bench_start = Instant::now();
    let mode = args.mode.clone();
    let hold_duration = Duration::from_secs(args.hold_secs);
    let stats = SharedStats::new();
    let total_connections = args.common.count as u64;
    let effective_concurrency = args.concurrency.min(args.common.count.max(1));
    let semaphore = Arc::new(Semaphore::new(effective_concurrency));
    let mut join_set = JoinSet::new();

    let progress_stats = stats.clone();
    let progress_handle = tokio::spawn(async move {
        loop {
            let snapshot = progress_stats.snapshot();
            let done = snapshot.success + snapshot.failed + snapshot.timeout >= total_connections;
            print_conn_progress_line(
                bench_start.elapsed(),
                total_connections,
                snapshot.success,
                snapshot.failed,
                done,
            );
            if done {
                return (bench_start.elapsed(), snapshot);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    for i in 0..args.common.count {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| BenchMarkError::ExecutionError(format!("semaphore closed: {e}")))?;
        let client_id = format!("robust-bench-conn-{i}");
        let host = args.common.host.clone();
        let port = args.common.port;
        let username = args.common.username.clone();
        let password = args.common.password.clone();
        let local_stats = stats.clone();
        let mode = mode.clone();
        let hold_duration_each = hold_duration;

        join_set.spawn(async move {
            let permit = permit;
            let start = Instant::now();
            let (client, mut event_loop) =
                build_client(&client_id, &host, port, &username, &password);
            match wait_connack(&mut event_loop, 60_000).await {
                Ok(_) => {
                    local_stats.incr_success();
                    local_stats.record_latency(start.elapsed());
                }
                Err(e) => {
                    local_stats.record_latency(start.elapsed());
                    if e.contains("connect timeout") {
                        local_stats.incr_timeout();
                    } else {
                        local_stats.incr_failed();
                    }
                    local_stats.record_error(&format!("connect:{e}"));
                    return;
                }
            }
            drop(permit);
            if matches!(mode, ConnMode::Hold) {
                let _ = tokio::time::timeout(hold_duration_each, async {
                    loop {
                        match event_loop.poll().await {
                            Ok(_) => {}
                            Err(_) => return,
                        }
                    }
                })
                .await;
                let _ = client.disconnect().await;
            }
        });
    }

    let (connect_phase_elapsed, snapshot) = progress_handle
        .await
        .map_err(|e| BenchMarkError::ExecutionError(format!("progress task failed: {e}")))?;
    let total_ops = snapshot.success;
    let connect_phase_secs = connect_phase_elapsed.as_secs_f64().max(0.001);
    let connect_qps = snapshot.success as f64 / connect_phase_secs;
    let report_duration_secs = connect_phase_elapsed.as_secs().max(1);
    print_realtime_line(
        "conn",
        connect_phase_elapsed,
        total_ops,
        total_ops,
        &snapshot,
    );
    println!(
        "[conn-core] connect_phase_secs={:.3} connect_qps={:.2}",
        connect_phase_secs, connect_qps
    );
    let series = vec![ThroughputSample {
        second: report_duration_secs,
        ops_per_sec: total_ops / report_duration_secs,
        total_ops,
        success: snapshot.success,
        failed: snapshot.failed,
        timeout: snapshot.timeout,
        received: snapshot.received,
    }];
    let mut extras = BTreeMap::new();
    extras.insert("mode".to_string(), format!("tcp-conn:{:?}", args.mode));
    extras.insert("concurrency".to_string(), effective_concurrency.to_string());
    extras.insert("hold_secs".to_string(), args.hold_secs.to_string());
    extras.insert("qos".to_string(), args.common.qos.to_string());

    let report = BenchReport::from_input(
        BenchReportInput {
            name: "mqtt-conn".to_string(),
            host: args.common.host,
            port: args.common.port,
            duration_secs: report_duration_secs,
            clients: args.common.count,
            op_label: "conn_ack".to_string(),
            total_ops,
            connect_phase_secs: Some(connect_phase_secs),
            connect_qps: Some(connect_qps),
            extras,
            series,
        },
        snapshot,
    );
    match args.common.output {
        OutputFormat::Table => report.print_table(),
        OutputFormat::Json => report.print_json(),
    }

    // Keep hold mode semantics: keep connections alive until all tasks finish.
    while join_set.join_next().await.is_some() {}
    Ok(())
}
