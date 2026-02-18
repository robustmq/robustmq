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
use crate::grpc::{
    PlacementCreateSessionArgs, PlacementDeleteSessionArgs, PlacementListSessionArgs,
};
use crate::mqtt::report::{print_realtime_line, BenchReport, BenchReportInput, ThroughputSample};
use crate::mqtt::stats::SharedStats;
use crate::mqtt::OutputFormat;
use grpc_clients::meta::mqtt::call::{
    placement_create_session, placement_delete_session, placement_list_session,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use protocol::meta::meta_service_mqtt::{
    CreateSessionRequest, DeleteSessionRequest, ListSessionRequest,
};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub async fn run_placement_create_session_bench(
    args: PlacementCreateSessionArgs,
) -> Result<(), BenchMarkError> {
    if args.count == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "count must be greater than 0".to_string(),
        ));
    }
    if args.concurrency == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "concurrency must be greater than 0".to_string(),
        ));
    }

    let bench_start = Instant::now();
    let total_requests = args.count as u64;
    let effective_concurrency = args.concurrency.min(args.count);
    let semaphore = Arc::new(Semaphore::new(effective_concurrency));
    let stats = SharedStats::new();
    let client_pool = Arc::new(ClientPool::new(effective_concurrency.min(8)));
    let addrs = vec![format!("{}:{}", args.host, args.port)];
    let timeout = Duration::from_millis(args.timeout_ms.max(1));

    let monitor_stats = stats.clone();
    let monitor = tokio::spawn(async move {
        let mut series = Vec::new();
        let mut prev_done = 0_u64;
        let monitor_start = Instant::now();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let success = monitor_stats.counters.success.load(Ordering::Relaxed);
            let failed = monitor_stats.counters.failed.load(Ordering::Relaxed);
            let timeout_count = monitor_stats.counters.timeout.load(Ordering::Relaxed);
            let done = success + failed + timeout_count;
            let delta = done.saturating_sub(prev_done);
            let snapshot = monitor_stats.snapshot();
            print_realtime_line(
                "meta/placement-create-session",
                monitor_start.elapsed(),
                delta,
                done,
                &snapshot,
            );
            series.push(ThroughputSample {
                second: monitor_start.elapsed().as_secs(),
                ops_per_sec: delta,
                total_ops: done,
                success: snapshot.success,
                failed: snapshot.failed,
                timeout: snapshot.timeout,
                received: snapshot.received,
            });
            prev_done = done;
            if done >= total_requests {
                break;
            }
        }

        series
    });

    let mut join_set = JoinSet::new();
    for i in 0..args.count {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| BenchMarkError::ExecutionError(format!("semaphore closed: {e}")))?;
        let local_stats = stats.clone();
        let local_pool = client_pool.clone();
        let local_addrs = addrs.clone();
        let client_id_prefix = args.client_id_prefix.clone();
        let session_expiry_secs = args.session_expiry_secs;

        join_set.spawn(async move {
            let _permit = permit;
            let client_id = format!("{client_id_prefix}-{}", i);
            let mut mqtt_session =
                MqttSession::new(client_id.clone(), session_expiry_secs, false, None, true);
            mqtt_session.update_broker_id(Some(1));
            mqtt_session.update_connection_id(Some((i + 1) as u64));
            let request = CreateSessionRequest {
                client_id,
                session: match mqtt_session.encode() {
                    Ok(data) => data,
                    Err(e) => {
                        local_stats.incr_failed();
                        local_stats.record_error(&format!("encode:{e}"));
                        return;
                    }
                },
            };

            let start = Instant::now();
            match tokio::time::timeout(
                timeout,
                placement_create_session(&local_pool, &local_addrs, request),
            )
            .await
            {
                Ok(Ok(_)) => {
                    local_stats.incr_success();
                    local_stats.record_latency(start.elapsed());
                }
                Ok(Err(e)) => {
                    local_stats.incr_failed();
                    local_stats.record_latency(start.elapsed());
                    local_stats.record_error(&format!("rpc:{e}"));
                }
                Err(_) => {
                    local_stats.incr_timeout();
                    local_stats.record_error("timeout");
                }
            }
        });
    }

    while join_set.join_next().await.is_some() {}
    let series = monitor.await.unwrap_or_default();
    let snapshot = stats.snapshot();
    let total_ops = snapshot.success + snapshot.failed + snapshot.timeout;
    let elapsed = bench_start.elapsed();
    let duration_secs = elapsed.as_secs().max(1);

    let mut extras = BTreeMap::new();
    extras.insert(
        "scenario".to_string(),
        "meta/placement-create-session".to_string(),
    );
    extras.insert("timeout_ms".to_string(), args.timeout_ms.to_string());
    extras.insert("concurrency".to_string(), effective_concurrency.to_string());
    extras.insert(
        "session_expiry_secs".to_string(),
        args.session_expiry_secs.to_string(),
    );

    let report = BenchReport::from_input(
        BenchReportInput {
            name: "meta-placement-create-session".to_string(),
            host: args.host,
            port: args.port,
            duration_secs,
            clients: args.count,
            op_label: "meta_call_complete".to_string(),
            total_ops,
            connect_phase_secs: None,
            connect_qps: None,
            extras,
            series,
        },
        snapshot,
    );

    match args.output {
        OutputFormat::Table => report.print_table(),
        OutputFormat::Json => report.print_json(),
    }

    Ok(())
}

pub async fn run_placement_list_session_bench(
    args: PlacementListSessionArgs,
) -> Result<(), BenchMarkError> {
    if args.count == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "count must be greater than 0".to_string(),
        ));
    }
    if args.concurrency == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "concurrency must be greater than 0".to_string(),
        ));
    }

    let client_pool = Arc::new(ClientPool::new(args.concurrency.min(args.count).min(8)));
    let addrs = vec![format!("{}:{}", args.host, args.port)];
    let timeout = Duration::from_millis(args.timeout_ms.max(1));

    // Setup phase: create one session so that the read bench has data to query.
    let setup_client_id = format!("{}-0", args.client_id_prefix);
    let mut mqtt_session = MqttSession::new(setup_client_id.clone(), 3600, false, None, true);
    mqtt_session.update_broker_id(Some(1));
    mqtt_session.update_connection_id(Some(1));
    let setup_request = CreateSessionRequest {
        client_id: setup_client_id.clone(),
        session: mqtt_session
            .encode()
            .map_err(|e| BenchMarkError::ExecutionError(format!("encode setup session: {e}")))?,
    };

    println!("[setup] Creating session: {setup_client_id}");
    match tokio::time::timeout(
        timeout,
        placement_create_session(&client_pool, &addrs, setup_request),
    )
    .await
    {
        Ok(Ok(_)) => println!("[setup] Session created successfully"),
        Ok(Err(e)) => {
            return Err(BenchMarkError::ExecutionError(format!(
                "setup create_session failed: {e}"
            )));
        }
        Err(_) => {
            return Err(BenchMarkError::ExecutionError(
                "setup create_session timed out".to_string(),
            ));
        }
    }

    // Bench phase: repeatedly list the same session.
    let bench_start = Instant::now();
    let total_requests = args.count as u64;
    let effective_concurrency = args.concurrency.min(args.count);
    let semaphore = Arc::new(Semaphore::new(effective_concurrency));
    let stats = SharedStats::new();
    let query_client_id = Arc::new(setup_client_id);

    let monitor_stats = stats.clone();
    let monitor = tokio::spawn(async move {
        let mut series = Vec::new();
        let mut prev_done = 0_u64;
        let monitor_start = Instant::now();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let success = monitor_stats.counters.success.load(Ordering::Relaxed);
            let failed = monitor_stats.counters.failed.load(Ordering::Relaxed);
            let timeout_count = monitor_stats.counters.timeout.load(Ordering::Relaxed);
            let done = success + failed + timeout_count;
            let delta = done.saturating_sub(prev_done);
            let snapshot = monitor_stats.snapshot();
            print_realtime_line(
                "meta/placement-list-session",
                monitor_start.elapsed(),
                delta,
                done,
                &snapshot,
            );
            series.push(ThroughputSample {
                second: monitor_start.elapsed().as_secs(),
                ops_per_sec: delta,
                total_ops: done,
                success: snapshot.success,
                failed: snapshot.failed,
                timeout: snapshot.timeout,
                received: snapshot.received,
            });
            prev_done = done;
            if done >= total_requests {
                break;
            }
        }

        series
    });

    let mut join_set = JoinSet::new();
    for _ in 0..args.count {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| BenchMarkError::ExecutionError(format!("semaphore closed: {e}")))?;
        let local_stats = stats.clone();
        let local_pool = client_pool.clone();
        let local_addrs = addrs.clone();
        let local_client_id = query_client_id.clone();

        join_set.spawn(async move {
            let _permit = permit;
            let request = ListSessionRequest {
                client_id: (*local_client_id).clone(),
            };

            let start = Instant::now();
            match tokio::time::timeout(
                timeout,
                placement_list_session(&local_pool, &local_addrs, request),
            )
            .await
            {
                Ok(Ok(reply)) => {
                    local_stats.incr_success();
                    local_stats.record_latency(start.elapsed());
                    local_stats.add_received(reply.sessions.len() as u64);
                }
                Ok(Err(e)) => {
                    local_stats.incr_failed();
                    local_stats.record_latency(start.elapsed());
                    local_stats.record_error(&format!("rpc:{e}"));
                }
                Err(_) => {
                    local_stats.incr_timeout();
                    local_stats.record_error("timeout");
                }
            }
        });
    }

    while join_set.join_next().await.is_some() {}
    let series = monitor.await.unwrap_or_default();
    let snapshot = stats.snapshot();
    let total_ops = snapshot.success + snapshot.failed + snapshot.timeout;
    let elapsed = bench_start.elapsed();
    let duration_secs = elapsed.as_secs().max(1);

    let mut extras = BTreeMap::new();
    extras.insert(
        "scenario".to_string(),
        "meta/placement-list-session".to_string(),
    );
    extras.insert("timeout_ms".to_string(), args.timeout_ms.to_string());
    extras.insert("concurrency".to_string(), effective_concurrency.to_string());

    let report = BenchReport::from_input(
        BenchReportInput {
            name: "meta-placement-list-session".to_string(),
            host: args.host,
            port: args.port,
            duration_secs,
            clients: args.count,
            op_label: "meta_call_complete".to_string(),
            total_ops,
            connect_phase_secs: None,
            connect_qps: None,
            extras,
            series,
        },
        snapshot,
    );

    match args.output {
        OutputFormat::Table => report.print_table(),
        OutputFormat::Json => report.print_json(),
    }

    Ok(())
}

pub async fn run_placement_delete_session_bench(
    args: PlacementDeleteSessionArgs,
) -> Result<(), BenchMarkError> {
    if args.count == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "count must be greater than 0".to_string(),
        ));
    }
    if args.concurrency == 0 {
        return Err(BenchMarkError::InvalidConfiguration(
            "concurrency must be greater than 0".to_string(),
        ));
    }

    let client_pool = Arc::new(ClientPool::new(args.concurrency.min(args.count).min(8)));
    let addrs = vec![format!("{}:{}", args.host, args.port)];
    let timeout = Duration::from_millis(args.timeout_ms.max(1));

    // Setup phase: create the session so the first delete has something to remove.
    // Subsequent deletes against a non-existent session still exercise the full Raft write path.
    let setup_client_id = args.client_id.clone();
    let mut mqtt_session = MqttSession::new(setup_client_id.clone(), 3600, false, None, true);
    mqtt_session.update_broker_id(Some(1));
    mqtt_session.update_connection_id(Some(1));
    let setup_request = CreateSessionRequest {
        client_id: setup_client_id.clone(),
        session: mqtt_session
            .encode()
            .map_err(|e| BenchMarkError::ExecutionError(format!("encode setup session: {e}")))?,
    };

    println!("[setup] Creating session: {setup_client_id}");
    match tokio::time::timeout(
        timeout,
        placement_create_session(&client_pool, &addrs, setup_request),
    )
    .await
    {
        Ok(Ok(_)) => println!("[setup] Session created successfully"),
        Ok(Err(e)) => {
            return Err(BenchMarkError::ExecutionError(format!(
                "setup create_session failed: {e}"
            )));
        }
        Err(_) => {
            return Err(BenchMarkError::ExecutionError(
                "setup create_session timed out".to_string(),
            ));
        }
    }

    // Bench phase: hammer delete_session with the fixed client_id.
    // After the first call the session no longer exists; all subsequent calls still go through
    // the full Raft write path on the Meta service, which is exactly what we want to measure.
    let bench_start = Instant::now();
    let total_requests = args.count as u64;
    let effective_concurrency = args.concurrency.min(args.count);
    let semaphore = Arc::new(Semaphore::new(effective_concurrency));
    let stats = SharedStats::new();
    let fixed_client_id = Arc::new(args.client_id.clone());

    let monitor_stats = stats.clone();
    let monitor = tokio::spawn(async move {
        let mut series = Vec::new();
        let mut prev_done = 0_u64;
        let monitor_start = Instant::now();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let success = monitor_stats.counters.success.load(Ordering::Relaxed);
            let failed = monitor_stats.counters.failed.load(Ordering::Relaxed);
            let timeout_count = monitor_stats.counters.timeout.load(Ordering::Relaxed);
            let done = success + failed + timeout_count;
            let delta = done.saturating_sub(prev_done);
            let snapshot = monitor_stats.snapshot();
            print_realtime_line(
                "meta/placement-delete-session",
                monitor_start.elapsed(),
                delta,
                done,
                &snapshot,
            );
            series.push(ThroughputSample {
                second: monitor_start.elapsed().as_secs(),
                ops_per_sec: delta,
                total_ops: done,
                success: snapshot.success,
                failed: snapshot.failed,
                timeout: snapshot.timeout,
                received: snapshot.received,
            });
            prev_done = done;
            if done >= total_requests {
                break;
            }
        }

        series
    });

    let mut join_set = JoinSet::new();
    for _ in 0..args.count {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| BenchMarkError::ExecutionError(format!("semaphore closed: {e}")))?;
        let local_stats = stats.clone();
        let local_pool = client_pool.clone();
        let local_addrs = addrs.clone();
        let local_client_id = fixed_client_id.clone();

        join_set.spawn(async move {
            let _permit = permit;
            let request = DeleteSessionRequest {
                client_id: (*local_client_id).clone(),
            };

            let start = Instant::now();
            match tokio::time::timeout(
                timeout,
                placement_delete_session(&local_pool, &local_addrs, request),
            )
            .await
            {
                Ok(Ok(_)) => {
                    local_stats.incr_success();
                    local_stats.record_latency(start.elapsed());
                }
                Ok(Err(e)) => {
                    local_stats.incr_failed();
                    local_stats.record_latency(start.elapsed());
                    local_stats.record_error(&format!("rpc:{e}"));
                }
                Err(_) => {
                    local_stats.incr_timeout();
                    local_stats.record_error("timeout");
                }
            }
        });
    }

    while join_set.join_next().await.is_some() {}
    let series = monitor.await.unwrap_or_default();
    let snapshot = stats.snapshot();
    let total_ops = snapshot.success + snapshot.failed + snapshot.timeout;
    let elapsed = bench_start.elapsed();
    let duration_secs = elapsed.as_secs().max(1);

    let mut extras = BTreeMap::new();
    extras.insert(
        "scenario".to_string(),
        "meta/placement-delete-session".to_string(),
    );
    extras.insert("timeout_ms".to_string(), args.timeout_ms.to_string());
    extras.insert("concurrency".to_string(), effective_concurrency.to_string());
    extras.insert("client_id".to_string(), args.client_id.clone());

    let report = BenchReport::from_input(
        BenchReportInput {
            name: "meta-placement-delete-session".to_string(),
            host: args.host,
            port: args.port,
            duration_secs,
            clients: args.count,
            op_label: "meta_call_complete".to_string(),
            total_ops,
            connect_phase_secs: None,
            connect_qps: None,
            extras,
            series,
        },
        snapshot,
    );

    match args.output {
        OutputFormat::Table => report.print_table(),
        OutputFormat::Json => report.print_json(),
    }

    Ok(())
}
