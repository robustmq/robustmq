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

use std::{sync::Arc, time::Duration};

use clap::Parser;
use dashmap::DashMap;
use futures::future;
use prettytable::{row, Table};
use protocol::placement_center::placement_center_kv::{GetRequest, SetRequest};
use rand::{thread_rng, Rng};
use serde::{de::DeserializeOwned, Serialize};
use tokio::time::Instant;

use crate::{error::BenchMarkError, BenchMark};
use grpc_clients::{
    placement::kv::call::{placement_get, placement_set},
    pool::ClientPool,
};

#[derive(Debug, Clone, Parser)]
pub struct KvGetBenchArgs {
    /// The number of concurrent clients to simulate
    #[clap(long, default_value = "100")]
    pub num_clients: usize,

    /// The number of worker threads to run the benchmark
    #[clap(long, default_value = "4")]
    pub worker_threads: usize,

    /// The number of keys to get
    #[clap(long, default_value = "10000")]
    pub num_keys: usize,

    /// The size of each value in bytes
    #[clap(long, default_value = "64")]
    pub value_size: usize,

    /// The address of the placement center
    #[clap(long, default_value = "127.0.0.1:1228", value_delimiter = ',')]
    pub pc_addrs: Vec<String>,
}

async fn do_placement_get<T>(
    client_pool: &ClientPool,
    addrs: &[&str],
    req: GetRequest,
) -> Result<T, BenchMarkError>
where
    T: DeserializeOwned,
{
    let reply = placement_get(client_pool, addrs, req)
        .await
        .map_err(Box::new)?;

    let ret = serde_json::from_str(reply.value.as_str())?;

    Ok(ret)
}

async fn do_placement_set<T>(
    client_pool: &ClientPool,
    addrs: &[&str],
    key: String,
    value: T,
) -> Result<(), BenchMarkError>
where
    T: Serialize,
{
    let req = SetRequest {
        key,
        value: serde_json::to_string(&value)?,
    };

    placement_set(client_pool, addrs, req)
        .await
        .map_err(Box::new)?;

    Ok(())
}

// generate random data of specified length, consisting of alphanumeric characters
fn gen_data(length: usize) -> String {
    let mut rng = thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();

    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

#[axum::async_trait]
impl BenchMark for KvGetBenchArgs {
    fn validate(&self) -> Result<(), BenchMarkError> {
        Ok(())
    }

    async fn do_bench(&self) -> Result<(), BenchMarkError> {
        self.validate()?;

        let KvGetBenchArgs {
            num_clients,
            worker_threads,
            num_keys,
            value_size,
            pc_addrs,
        } = self.clone();

        println!(
            "Starting KV Get Benchmark with {} clients, {} worker threads, {} keys, value size: {}, placement center addresses: {:?}",
            num_clients, worker_threads, num_keys, value_size, pc_addrs
        );

        let client_pool = Arc::new(ClientPool::new(num_clients as u64));
        let mut set_handles = Vec::with_capacity(num_clients);

        for client_id in 0..num_clients {
            let client_pool_shared = client_pool.clone();
            let pc_addrs_clone = pc_addrs.clone();
            let set_handle = tokio::spawn(async move {
                let result: Result<(), BenchMarkError> = async {
                    for key_id in 0..num_keys {
                        // generate random key and value
                        let key = (client_id * num_keys + key_id).to_string();

                        let value = gen_data(value_size);

                        do_placement_set(
                            &client_pool_shared,
                            &pc_addrs_clone
                                .iter()
                                .map(String::as_str)
                                .collect::<Vec<_>>(),
                            key,
                            value,
                        )
                        .await?;
                    }
                    Ok(())
                }
                .await;
                result
            });

            set_handles.push(set_handle);
        }

        future::join_all(set_handles).await;

        let mut get_handles = Vec::with_capacity(num_clients);

        // we start the timer after all kv pairs are set
        let latencies = Arc::new(DashMap::with_capacity(num_clients));

        let total_now = Instant::now();

        for client_id in 0..num_clients {
            let client_pool_shared = client_pool.clone();
            let latencies_shared = latencies.clone();
            let pc_addrs_clone = pc_addrs.clone();
            let get_handle = tokio::spawn(async move {
                let result: Result<(), BenchMarkError> = async {
                    for key_id in 0..num_keys {
                        // generate random key and value
                        let key = (client_id * num_keys + key_id).to_string();

                        let req = GetRequest { key };

                        let now = Instant::now();

                        // get the value back
                        do_placement_get::<String>(
                            &client_pool_shared,
                            &pc_addrs_clone
                                .iter()
                                .map(String::as_str)
                                .collect::<Vec<_>>(),
                            req,
                        )
                        .await?;

                        latencies_shared
                            .entry(client_id)
                            .and_modify(|e: &mut Vec<Duration>| {
                                e.push(now.elapsed());
                            })
                            .or_insert_with(|| vec![now.elapsed()]);
                    }
                    Ok(())
                }
                .await;
                result
            });

            get_handles.push(get_handle);
        }

        future::join_all(get_handles).await;

        let total_time = total_now.elapsed();

        println!(
            "Benchmark KV get finished with time: {:?}s, ms/op: {}, op/ms: {}",
            total_time.as_secs_f64(),
            total_time.as_millis() as f64 / (num_clients * num_keys) as f64,
            (num_clients * num_keys) as f64 / total_time.as_millis() as f64,
        );

        // Sort latencies by client ID
        let mut latencies_sorted = latencies
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect::<Vec<_>>();

        latencies_sorted.sort_by_key(|(client_id, _)| *client_id);

        // Print the results in a table format
        let mut table = Table::new();

        table.add_row(row![
            "Client ID",
            "P50 Latency (us)",
            "P90 Latency (us)",
            "P99 Latency (us)",
            "P999 Latency (us)",
            "Average Latency (us)",
            "Min Latency (us)",
            "Max Latency (us)",
            "Median Latency (us)"
        ]);

        for (client_id, latencies) in latencies_sorted.iter() {
            let mut sorted_latencies = latencies.clone();
            sorted_latencies.sort_unstable();

            let p50 = sorted_latencies[sorted_latencies.len() / 2].as_micros();
            let p90 = sorted_latencies[(sorted_latencies.len() * 9) / 10].as_micros();
            let p99 = sorted_latencies[(sorted_latencies.len() * 99) / 100].as_micros();
            let p999 = sorted_latencies[(sorted_latencies.len() * 999) / 1000].as_micros();
            let average: u128 = sorted_latencies.iter().map(|d| d.as_micros()).sum::<u128>()
                / sorted_latencies.len() as u128;
            let min = sorted_latencies.first().unwrap().as_micros();
            let max = sorted_latencies.last().unwrap().as_micros();
            let median = sorted_latencies[sorted_latencies.len() / 2].as_micros();

            table.add_row(row![
                client_id, p50, p90, p99, p999, average, min, max, median
            ]);
        }

        table.printstd();

        Ok(())
    }
}
