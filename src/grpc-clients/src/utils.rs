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

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use common_base::error::common::CommonError;
use common_metrics::grpc::record_grpc_client_call;
use regex::Regex;
use tokio::time::sleep;

use crate::pool::ClientPool;
use crate::retry_times;

pub(crate) trait RetriableRequest: Clone {
    type Client;
    type Response;
    type Error: std::error::Error;

    const IS_WRITE_REQUEST: bool = false;

    fn method_name() -> &'static str;

    fn get_client(pool: &ClientPool, addr: &str) -> Self::Client;

    async fn call_once(
        client: &mut Self::Client,
        request: Self,
    ) -> Result<Self::Response, Self::Error>;
}

pub(crate) async fn retry_call<Req>(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: Req,
) -> Result<Req::Response, CommonError>
where
    Req: RetriableRequest,
    Req::Error: Into<CommonError>,
{
    let start = Instant::now();
    let method = Req::method_name();
    let result = retry_call_inner::<Req>(client_pool, addrs, request).await;
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    let (service, method_name) = method.split_once('/').unwrap_or(("unknown", method));
    record_grpc_client_call(service, method_name, duration_ms);

    result
}

async fn retry_call_inner<Req>(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: Req,
) -> Result<Req::Response, CommonError>
where
    Req: RetriableRequest,
    Req::Error: Into<CommonError>,
{
    if addrs.is_empty() {
        return Err(CommonError::CommonError(
            "Call address list cannot be empty".to_string(),
        ));
    }

    let method = Req::method_name();
    // A few quick retries to ride out transient failures (leader briefly busy,
    // a peer still starting up) without exceeding callers' own timeouts — e.g.
    // heartbeat wraps this in a 3s timeout, so total retry time must stay well
    // under that. Callers needing longer recovery (e.g. node re-registration)
    // retry at their own layer.
    const MIN_ATTEMPTS: usize = 3;
    const RETRY_INTERVAL_MS: u64 = 100;
    let mut times = 0;
    loop {
        let index = times % addrs.len();
        times += 1;
        // For write requests, always prefer the cached leader if present; only
        // fall back to round-robin when there is no cached leader. We do NOT
        // give up on the leader just because one attempt failed — it may be
        // briefly unavailable (committing a membership change, restarting, etc.).
        let target_addr = if Req::IS_WRITE_REQUEST {
            client_pool
                .get_leader_addr(method)
                .map(|l| l.value().to_string())
                .unwrap_or_else(|| addrs[index].as_ref().to_string())
        } else {
            addrs[index].as_ref().to_string()
        };

        let mut client = Req::get_client(client_pool, &target_addr);

        match Req::call_once(&mut client, request.clone()).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                let err: CommonError = e.into();

                if err.to_string().contains("forward request to") {
                    // Not the leader — follow the redirect and cache the real leader.
                    if let Some(leader_addr) = get_forward_addr(&err) {
                        client_pool.set_leader_addr(method.to_string(), leader_addr.clone());
                        let mut leader_client = Req::get_client(client_pool, &leader_addr);
                        if let Ok(data) = Req::call_once(&mut leader_client, request.clone()).await
                        {
                            return Ok(data);
                        }
                    }
                }
                if times >= MIN_ATTEMPTS.max(retry_times()) {
                    return Err(err);
                }
                sleep(Duration::from_millis(RETRY_INTERVAL_MS)).await;
            }
        }
    }
}

pub fn get_forward_addr(err: &CommonError) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"rpc_addr: ([^}]+)").unwrap());

    let error_info = err.to_string();
    let raw = re.captures(&error_info)?.get(1)?.as_str();
    Some(raw.replace(['\\', '"', ' '], ""))
}
