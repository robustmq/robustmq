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
use crate::mqtt::common::{build_client, qos_from_u8, wait_connack};
use crate::mqtt::report::{print_realtime_line, BenchReport, BenchReportInput, ThroughputSample};
use crate::mqtt::stats::SharedStats;
use crate::mqtt::{OutputFormat, PublishBenchArgs};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

pub async fn run_publish_bench(args: PublishBenchArgs) -> Result<(), BenchMarkError> {
    let duration = args.common.duration();
    let deadline = tokio::time::Instant::now() + duration;
    let qos = qos_from_u8(args.common.qos);
    let payload = vec![b'x'; args.payload_size];
    let stats = SharedStats::new();
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(args.common.count);

    for i in 0..args.common.count {
        if args.common.interval_ms > 0 {
            tokio::time::sleep(Duration::from_millis(args.common.interval_ms)).await;
        }

        let local_stats = stats.clone();
        let host = args.common.host.clone();
        let port = args.common.port;
        let username = args.common.username.clone();
        let password = args.common.password.clone();
        let topic = args.topic.replace("%i", &i.to_string());
        let payload = payload.clone();
        let publish_interval = Duration::from_millis(args.message_interval_ms.max(1));

        handles.push(tokio::spawn(async move {
            let client_id = format!("robust-bench-pub-{i}");
            let (client, mut event_loop) =
                build_client(&client_id, &host, port, &username, &password);
            if let Err(e) = wait_connack(&mut event_loop, 10_000).await {
                local_stats.incr_failed();
                local_stats.record_error(&format!("connect:{e}"));
                return;
            }

            let mut next_publish = tokio::time::Instant::now();
            while tokio::time::Instant::now() < deadline {
                let now = tokio::time::Instant::now();
                if now >= next_publish {
                    let start = Instant::now();
                    match client
                        .publish(topic.clone(), qos, false, payload.clone())
                        .await
                    {
                        Ok(_) => {
                            local_stats.incr_success();
                            local_stats.record_latency(start.elapsed());
                        }
                        Err(e) => {
                            local_stats.incr_failed();
                            local_stats.record_error(&format!("publish:{e}"));
                        }
                    }
                    next_publish += publish_interval;
                }

                if let Ok(Err(e)) =
                    tokio::time::timeout(Duration::from_millis(5), event_loop.poll()).await
                {
                    local_stats.record_error(&format!("poll:{e}"));
                }
            }
            let _ = client.disconnect().await;
        }));
    }

    let monitor_stats = stats.clone();
    let monitor = tokio::spawn(async move {
        let mut series = Vec::new();
        let mut prev_ok = 0;
        let begin = Instant::now();
        while tokio::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let current = monitor_stats
                .counters
                .success
                .load(std::sync::atomic::Ordering::Relaxed);
            let delta = current.saturating_sub(prev_ok);
            let snapshot = monitor_stats.snapshot();
            print_realtime_line("pub", begin.elapsed(), delta, current, &snapshot);
            series.push(ThroughputSample {
                second: begin.elapsed().as_secs(),
                ops_per_sec: delta,
                total_ops: current,
                success: snapshot.success,
                failed: snapshot.failed,
                timeout: snapshot.timeout,
                received: snapshot.received,
            });
            prev_ok = current;
        }
        series
    });

    for handle in handles {
        let _ = handle.await;
    }
    let series = monitor.await.unwrap_or_default();
    let snapshot = stats.snapshot();
    let total_ops = snapshot.success;
    let mut extras = BTreeMap::new();
    extras.insert("mode".to_string(), "tcp-pub".to_string());
    extras.insert("topic".to_string(), args.topic);
    extras.insert("payload_size".to_string(), args.payload_size.to_string());
    extras.insert(
        "message_interval_ms".to_string(),
        args.message_interval_ms.to_string(),
    );
    extras.insert("qos".to_string(), args.common.qos.to_string());
    extras.insert(
        "interval_ms".to_string(),
        args.common.interval_ms.to_string(),
    );

    let report = BenchReport::from_input(
        BenchReportInput {
            name: "mqtt-pub".to_string(),
            host: args.common.host,
            port: args.common.port,
            duration_secs: duration.as_secs(),
            clients: args.common.count,
            op_label: "publish_success".to_string(),
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
