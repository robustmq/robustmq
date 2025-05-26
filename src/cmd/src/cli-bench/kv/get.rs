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

use std::{env, path::PathBuf, process::Command, sync::Arc};

use clap::Parser;
use futures::future;
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
}

fn get_script_path(script_name: &str) -> Option<PathBuf> {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(project_root) = exe_path
            .ancestors()
            .find(|path| path.join("scripts").exists())
        {
            return Some(project_root.join("scripts").join(script_name));
        }
    }
    None
}

fn start_pc_with_default_conf() -> Result<(), BenchMarkError> {
    let mut child = Command::new("bash")
        .arg(get_script_path("start-place.sh").unwrap())
        .spawn()?;

    child.wait()?;
    Ok(())
}

fn stop_pc() -> Result<(), BenchMarkError> {
    let mut child = Command::new("bash")
        .arg(get_script_path("stop-place.sh").unwrap())
        .spawn()?;

    child.wait()?;
    Ok(())
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
        } = self.clone();

        println!(
            "Starting KV Get Benchmark with {} clients, {} worker threads, {} keys, value size: {}",
            num_clients, worker_threads, num_keys, value_size
        );

        let total_key_size = num_clients
            * ((0..num_keys * num_clients)
                .map(|i| (i.to_string().len()))
                .sum::<usize>());

        let total_value_size = num_clients * num_keys * value_size;

        // set up placement kv server
        start_pc_with_default_conf()?;

        let client_pool = Arc::new(ClientPool::new(num_clients as u64));
        let mut set_handles = Vec::with_capacity(num_clients);

        for client_id in 0..num_clients {
            let client_pool = client_pool.clone();
            let set_handle = tokio::spawn(async move {
                let result: Result<(), BenchMarkError> = async {
                    for key_id in 0..num_keys {
                        // generate random key and value
                        let key = (client_id * num_keys + key_id).to_string();

                        let value = gen_data(value_size);

                        let pc_addrs = vec!["127.0.0.1:1228"];

                        do_placement_set(&client_pool, &pc_addrs, key, value).await?;
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
        let now = Instant::now();

        for client_id in 0..num_clients {
            let client_pool = client_pool.clone();
            let get_handle = tokio::spawn(async move {
                let result: Result<(), BenchMarkError> = async {
                    for key_id in 0..num_keys {
                        // generate random key and value
                        let key = (client_id * num_keys + key_id).to_string();

                        let pc_addrs = vec!["127.0.0.1:1228"];

                        let req = GetRequest { key };

                        // get the value back
                        let val = do_placement_get::<String>(&client_pool, &pc_addrs, req).await?;

                        assert!(!val.is_empty());
                    }
                    Ok(())
                }
                .await;
                result
            });

            get_handles.push(get_handle);
        }

        future::join_all(get_handles).await;

        let elapsed = now.elapsed();

        let total_size = total_key_size + total_value_size;

        println!(
            "Benchmark KV get finished with time: {:?}, ns/op: {}, op/ms: {}",
            elapsed,
            elapsed.as_nanos() / (total_size as u128),
            (total_size as u128) / elapsed.as_millis(),
        );

        // stop placement center
        stop_pc()?;

        println!("Placement Center stopped successfully.");

        Ok(())
    }
}
