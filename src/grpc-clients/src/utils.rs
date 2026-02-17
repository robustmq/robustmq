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

use std::collections::HashSet;
use std::time::Duration;

use common_base::error::common::CommonError;
use regex::Regex;
use tokio::time::sleep;

use crate::pool::ClientPool;
use crate::{retry_sleep_time, retry_times};

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
    if addrs.is_empty() {
        return Err(CommonError::CommonError(
            "Call address list cannot be empty".to_string(),
        ));
    }

    let method = Req::method_name();
    let mut times = 1;
    let mut tried_addrs = HashSet::new();
    loop {
        let index = times % addrs.len();
        let addr = addrs[index].as_ref();
        let target_addr = if Req::IS_WRITE_REQUEST {
            client_pool
                .get_leader_addr(method)
                .map(|leader| leader.value().to_string())
                .unwrap_or_else(|| addr.to_string())
        } else {
            addr.to_string()
        };
        if tried_addrs.contains(&target_addr) {
            if times > retry_times() {
                return Err(CommonError::CommonError("Not found leader".to_string()));
            }
            times += 1;
            continue;
        }

        let mut client = Req::get_client(client_pool, &target_addr);

        match Req::call_once(&mut client, request.clone()).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                let err: CommonError = e.into();

                if err.to_string().contains("forward request to") {
                    tried_addrs.insert(target_addr);

                    if let Some(leader_addr) = get_forward_addr(&err) {
                        client_pool.set_leader_addr(method.to_string(), leader_addr.clone());

                        if !tried_addrs.contains(&leader_addr) {
                            let mut leader_client = Req::get_client(client_pool, &leader_addr);

                            match Req::call_once(&mut leader_client, request.clone()).await {
                                Ok(data) => return Ok(data),
                                Err(_) => {
                                    tried_addrs.insert(leader_addr);
                                }
                            }
                        }
                    }
                } else {
                    return Err(err);
                }

                if times > retry_times() {
                    return Err(CommonError::CommonError("Not found leader".to_string()));
                }
                times += 1;
                sleep(Duration::from_secs(retry_sleep_time(times))).await;
            }
        }
    }
}

pub fn get_forward_addr(err: &CommonError) -> Option<String> {
    let error_info = err.to_string();
    let re = Regex::new(r"rpc_addr: ([^}]+)").unwrap();
    if let Some(caps) = re.captures(&error_info) {
        if let Some(rpc_addr) = caps.get(1) {
            let mut leader_addr = rpc_addr.as_str().to_string();
            leader_addr = leader_addr.replace("\\", "");
            leader_addr = leader_addr.replace("\"", "");
            leader_addr = leader_addr.replace(" ", "");
            return Some(leader_addr);
        }
    }
    None
}
