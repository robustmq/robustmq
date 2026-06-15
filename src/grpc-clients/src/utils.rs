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
use tracing::{debug, info, warn};

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
    // Per-attempt timeout: prevents a node that is reachable at TCP level but
    // not responding (e.g. installing a Raft snapshot) from blocking the entire
    // retry loop.  On timeout the error is treated as a transport error so the
    // loop continues to the next address.
    const PER_CALL_TIMEOUT: Duration = Duration::from_secs(5);
    // Try every node at least once before giving up: a write may be pinned to a
    // stale cached leader, and the live leader could be any other node.
    let max_attempts = MIN_ATTEMPTS.max(retry_times()).max(addrs.len());
    let mut times = 0;
    loop {
        let index = times % addrs.len();
        times += 1;
        // Write requests prefer the cached leader when present, falling back to
        // round-robin otherwise. A transport failure against the cached leader
        // drops it (see below) so a crashed / re-elected leader is re-discovered.
        let (target_addr, source) = if Req::IS_WRITE_REQUEST {
            match client_pool.get_leader_addr(method) {
                Some(l) => (l.value().to_string(), "cached-leader"),
                None => (addrs[index].as_ref().to_string(), "round-robin"),
            }
        } else {
            (addrs[index].as_ref().to_string(), "round-robin")
        };

        debug!(
            "retry_call {} attempt {}/{}: target {} (via {}, index {})",
            method, times, max_attempts, target_addr, source, index
        );

        let mut client = Req::get_client(client_pool, &target_addr);

        let raw = tokio::time::timeout(
            PER_CALL_TIMEOUT,
            Req::call_once(&mut client, request.clone()),
        )
        .await;
        let err: CommonError = match raw {
            Ok(Ok(data)) => return Ok(data),
            Ok(Err(e)) => e.into(),
            Err(_elapsed) => {
                warn!(
                    "retry_call {} attempt {}/{}: {} did not respond within {:?}",
                    method, times, max_attempts, target_addr, PER_CALL_TIMEOUT
                );
                // Treated as a transport error so the loop continues to the next address.
                CommonError::CommonError(format!(
                    "tcp connect error: {} timed out after {:?}",
                    target_addr, PER_CALL_TIMEOUT
                ))
            }
        };
        if err.to_string().contains("forward request to") {
            // Not the leader — follow the redirect and cache the real leader.
            if let Some(leader_addr) = get_forward_addr(&err) {
                info!(
                    "retry_call {} attempt {}: {} redirected to leader {}",
                    method, times, target_addr, leader_addr
                );
                client_pool.set_leader_addr(method.to_string(), leader_addr.clone());
                let mut leader_client = Req::get_client(client_pool, &leader_addr);
                match Req::call_once(&mut leader_client, request.clone()).await {
                    Ok(data) => return Ok(data),
                    Err(le) => {
                        let le: CommonError = le.into();
                        if is_transport_error(&le) {
                            // The redirected leader is unreachable — drop it
                            // so the next attempt sweeps the node list and
                            // re-discovers it.
                            warn!(
                                "retry_call {} attempt {}: redirected leader {} unreachable: {}",
                                method, times, leader_addr, le
                            );
                            client_pool.remove_leader_addr(method);
                        } else {
                            // The leader processed and rejected the request
                            // (application error) — authoritative, return now.
                            warn!(
                                "retry_call {} attempt {}: redirected leader {} rejected the request (not retried): {}",
                                method, times, leader_addr, le
                            );
                            return Err(le);
                        }
                    }
                }
            } else {
                warn!(
                    "retry_call {} attempt {}: {} returned a forward error but no leader addr parsed: {}",
                    method, times, target_addr, err
                );
            }
        } else if is_transport_error(&err) {
            // The node is unreachable (down / not yet listening) — sweep on
            // to the next node.
            warn!(
                "retry_call {} attempt {}: {} unreachable: {}",
                method, times, target_addr, err
            );
            if Req::IS_WRITE_REQUEST {
                // A write failed against the cached leader (e.g. it crashed
                // or a new leader was elected). Drop the stale leader so the
                // next attempt round-robins the nodes and re-discovers the
                // leader, instead of pinning to the same dead address.
                client_pool.remove_leader_addr(method);
            }
        } else {
            // The node responded and rejected the request for an
            // application reason (e.g. "not enough nodes"). This is an
            // authoritative answer — return it immediately instead of
            // masking it by retrying into unreachable nodes.
            warn!(
                "retry_call {} attempt {}: {} rejected the request (not retried): {}",
                method, times, target_addr, err
            );
            return Err(err);
        }
        if times >= max_attempts {
            return Err(err);
        }
        sleep(Duration::from_millis(RETRY_INTERVAL_MS)).await;
    }
}

/// Whether the error is a transport/availability failure (the node is
/// unreachable), as opposed to an application-level rejection. Only transport
/// failures are worth retrying against other nodes; an application rejection is
/// authoritative and should be surfaced immediately.
fn is_transport_error(err: &CommonError) -> bool {
    let s = err.to_string();
    s.contains("tcp connect error")
        || s.contains("Connection refused")
        || s.contains("ConnectError")
        || s.contains("The service is currently unavailable")
}

pub fn get_forward_addr(err: &CommonError) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"rpc_addr: ([^}]+)").unwrap());

    let error_info = err.to_string();
    let raw = re.captures(&error_info)?.get(1)?.as_str();
    Some(raw.replace(['\\', '"', ' '], ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock write request: `reject_addr` returns an application-level error,
    /// `good_addr` succeeds, every other address fails as if connection-refused.
    /// The client is just the target address; shared state records the call order.
    struct MockState {
        calls: Mutex<Vec<String>>,
        good_addr: String,
        reject_addr: Option<String>,
    }

    #[derive(Clone)]
    struct MockReq {
        state: Arc<MockState>,
    }

    impl RetriableRequest for MockReq {
        type Client = String;
        type Response = ();
        type Error = CommonError;

        const IS_WRITE_REQUEST: bool = true;

        fn method_name() -> &'static str {
            "MockService/MockWrite"
        }

        fn get_client(_pool: &ClientPool, addr: &str) -> Self::Client {
            addr.to_string()
        }

        async fn call_once(
            client: &mut Self::Client,
            request: Self,
        ) -> Result<Self::Response, Self::Error> {
            request.state.calls.lock().unwrap().push(client.clone());
            if request.state.reject_addr.as_deref() == Some(client.as_str()) {
                // Application-level rejection (not a transport failure).
                Err(CommonError::CommonError(
                    "There are not enough nodes available in the cluster".to_string(),
                ))
            } else if *client == request.state.good_addr {
                Ok(())
            } else {
                Err(CommonError::CommonError("tcp connect error".to_string()))
            }
        }
    }

    // A stale cached leader that is unreachable must be dropped so the retry
    // sweeps the node list and re-discovers a live node, instead of pinning to
    // the dead address every attempt.
    #[tokio::test]
    async fn write_drops_stale_leader_and_round_robins() {
        let pool = ClientPool::new(1);
        pool.set_leader_addr(
            MockReq::method_name().to_string(),
            "127.0.0.1:9999".to_string(),
        );

        let state = Arc::new(MockState {
            calls: Mutex::new(Vec::new()),
            good_addr: "127.0.0.1:2228".to_string(),
            reject_addr: None,
        });
        let addrs = ["127.0.0.1:1228", "127.0.0.1:2228", "127.0.0.1:3228"];
        let res = retry_call_inner::<MockReq>(
            &pool,
            &addrs,
            MockReq {
                state: state.clone(),
            },
        )
        .await;

        assert!(
            res.is_ok(),
            "should succeed once the dead cached leader is dropped"
        );
        let calls = state.calls.lock().unwrap();
        assert_eq!(
            calls[0], "127.0.0.1:9999",
            "first attempt uses the cached leader"
        );
        assert!(
            calls[1..].iter().any(|a| a == "127.0.0.1:2228"),
            "subsequent attempts round-robin to a live node, got {calls:?}"
        );
        // The stale leader must have been evicted from the cache.
        assert!(pool.get_leader_addr(MockReq::method_name()).is_none());
    }

    // With more nodes than the minimum attempt count, every node must still be
    // tried before failing — here the only live node is the last one.
    #[tokio::test]
    async fn tries_every_node_before_succeeding() {
        let pool = ClientPool::new(1);
        let state = Arc::new(MockState {
            calls: Mutex::new(Vec::new()),
            good_addr: "127.0.0.1:5005".to_string(),
            reject_addr: None,
        });
        let addrs = [
            "127.0.0.1:5001",
            "127.0.0.1:5002",
            "127.0.0.1:5003",
            "127.0.0.1:5004",
            "127.0.0.1:5005",
        ];
        let res = retry_call_inner::<MockReq>(
            &pool,
            &addrs,
            MockReq {
                state: state.clone(),
            },
        )
        .await;

        assert!(
            res.is_ok(),
            "should reach the last (only live) node, got {res:?}"
        );
        let calls = state.calls.lock().unwrap();
        assert!(
            calls.iter().any(|a| a == "127.0.0.1:5005"),
            "the last node must be tried, got {calls:?}"
        );
    }

    #[test]
    fn get_forward_addr_parses_and_strips() {
        let err = CommonError::CommonError(
            "has to forward request to: Some(Node { node_id: 2, rpc_addr: \"127.0.0.1:2228\" })"
                .to_string(),
        );
        assert_eq!(get_forward_addr(&err).as_deref(), Some("127.0.0.1:2228"));

        // No rpc_addr in the message → None.
        let plain = CommonError::CommonError("connection refused".to_string());
        assert_eq!(get_forward_addr(&plain), None);
    }

    // Only network errors and leader redirects are retried; an application-level
    // rejection (e.g. "not enough nodes") is authoritative and returned at once,
    // without sweeping the other nodes.
    #[tokio::test]
    async fn application_rejection_is_returned_immediately() {
        let pool = ClientPool::new(1);
        let state = Arc::new(MockState {
            calls: Mutex::new(Vec::new()),
            // :2228 would succeed, but it must never be reached.
            good_addr: "127.0.0.1:2228".to_string(),
            reject_addr: Some("127.0.0.1:1228".to_string()),
        });
        let addrs = ["127.0.0.1:1228", "127.0.0.1:2228", "127.0.0.1:3228"];
        let res = retry_call_inner::<MockReq>(
            &pool,
            &addrs,
            MockReq {
                state: state.clone(),
            },
        )
        .await;

        assert!(
            res.is_err(),
            "an application rejection must propagate as an error"
        );
        let calls = state.calls.lock().unwrap();
        assert_eq!(
            calls.as_slice(),
            ["127.0.0.1:1228"],
            "must not retry other nodes after an application rejection, got {calls:?}"
        );
    }
}
