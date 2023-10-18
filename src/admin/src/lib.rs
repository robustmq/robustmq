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

use std::{net::SocketAddr, str::FromStr, thread::{self, JoinHandle}};
use rlog;
use axum::{routing::get, Router};

const ROUTE_ROOT: &str = "/";
const ROUTE_METRICS: &str = "/metrics";

mod prometheus;
mod welcome;

pub fn start(addr: String, port: Option<u16>, worker_threads: usize) -> JoinHandle<()>{
    let handle = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .worker_threads(worker_threads)
            .max_blocking_threads(2048)
            .thread_name("admin-http")
            .enable_io()
            .build()
            .unwrap();

        runtime.block_on(async {
            let app = Router::new()
                .route(ROUTE_METRICS, get(prometheus::handler))
                .route(ROUTE_ROOT, get(welcome::handler));

            let ip = SocketAddr::from_str(&format!("{}:{}", addr, port.unwrap())).unwrap();
            rlog::info(&format!("http server start success. bind:{}",ip.to_string()));
            axum::Server::bind(&ip)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
    });
   return handle;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
