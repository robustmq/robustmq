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

use std::future::Future;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use common_base::error::common::CommonError;
use log::error;
use tokio::time::sleep;

use crate::pool::ClientPool;
use crate::{retry_sleep_time, retry_times};

pub(crate) trait RetriableRequest: Clone {
    type Client;
    type Response;
    type Error: std::error::Error;
    
    async fn get_client(pool: &ClientPool, addr: SocketAddr) -> Result<impl DerefMut<Target = Self::Client>, Self::Error>;

    async fn call_once(client: &mut Self::Client, request: Self) -> Result<Self::Response, Self::Error>;
}

pub(crate) async fn retry_call<'a, Req>(
    client_pool: &'a ClientPool,
    addrs: &'a [SocketAddr],
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

    let mut times = 1;
    loop {
        let index = times % addrs.len();
        let addr = addrs[index];
        let mut client = Req::get_client(client_pool, addr).await
            .map_err(Into::into)?;
        let result = Req::call_once(client.deref_mut(), request.clone()).await;

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                error!("{}", e);
                if times > retry_times() {
                    return Err(e.into());
                }
                times += 1;
            }
        }

        sleep(Duration::from_secs(retry_sleep_time(times))).await;
    }
}
