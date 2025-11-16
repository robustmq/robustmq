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

use std::time::Duration;

use dashmap::DashMap;
use grpc_clients::{
    meta::common::call::{kv_get, kv_set},
    pool::ClientPool,
};
use prettytable::{row, Table};
use protocol::meta::meta_service_common::{GetRequest, SetRequest};
use rand::{thread_rng, Rng};
use serde::{de::DeserializeOwned, Serialize};

use crate::BenchMarkError;

pub async fn do_placement_get<T>(
    client_pool: &ClientPool,
    addrs: &[&str],
    key: String,
) -> Result<T, BenchMarkError>
where
    T: DeserializeOwned,
{
    let req = GetRequest { key };
    let reply = kv_get(client_pool, addrs, req).await.map_err(Box::new)?;

    let ret = serde_json::from_str(&reply.value)?;

    Ok(ret)
}

pub async fn do_placement_set<T>(
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

    kv_set(client_pool, addrs, req).await.map_err(Box::new)?;

    Ok(())
}

// generate random data of specified length, consisting of alphanumeric characters
pub fn gen_data(length: usize) -> String {
    let mut rng = thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();

    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

pub fn sort_lantencies_by_client_id(
    latencies: &DashMap<usize, Vec<Duration>>,
) -> Vec<(usize, Vec<Duration>)> {
    let mut latencies_sorted = latencies
        .iter()
        .map(|entry| (*entry.key(), entry.value().clone()))
        .collect::<Vec<_>>();

    latencies_sorted.sort_by_key(|(client_id, _)| *client_id);
    latencies_sorted
}

pub fn print_total_statistics(total_time: Duration, op: &str, num_clients: usize, num_keys: usize) {
    println!(
        "==================\nBenchmark KV {} finished with:\n\t\ttime: {:?}s\n\t\tms/op: {}\n\t\top/ms: {}",
        op,
        total_time.as_secs_f64(),
        total_time.as_millis() as f64 / (num_clients * num_keys) as f64,
        (num_clients * num_keys) as f64 / total_time.as_millis() as f64,
    );
}

pub fn print_latency_table(all_latencies: &[(usize, Vec<Duration>)]) {
    let mut table_latency = Table::new();

    table_latency.add_row(row![
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

    for (client_id, latencies) in all_latencies.iter() {
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

        table_latency.add_row(row![
            client_id, p50, p90, p99, p999, average, min, max, median
        ]);
    }

    table_latency.printstd();
}

pub fn print_throughput_table(
    all_latencies: &[(usize, Vec<Duration>)],
    num_keys: usize,
    value_size: usize,
    total_key_sizes: &DashMap<usize, usize>,
) {
    let mut table_throughput = Table::new();

    table_throughput.add_row(row![
        "Client ID",
        "nMsg per second",
        "Total message size (bytes)",
        "Bytes per second"
    ]);

    for (client_id, latencies) in all_latencies.iter() {
        let mut sorted_latencies = latencies.clone();
        sorted_latencies.sort_unstable();

        let n_message = 2 * num_keys;

        let total_time_secs = sorted_latencies
            .iter()
            .map(|d| d.as_secs_f64())
            .sum::<f64>();

        let n_message_per_second = n_message as f64 / total_time_secs;

        let total_key_size = total_key_sizes
            .get(client_id)
            .map(|size| *size.value())
            .expect("Client Id not found");

        let total_value_size = num_keys * value_size;

        let bytes_per_second = (total_key_size + total_value_size) as f64 / total_time_secs;

        table_throughput.add_row(row![
            client_id,
            n_message_per_second,
            total_key_size + total_value_size,
            bytes_per_second
        ]);
    }

    table_throughput.printstd();
}
