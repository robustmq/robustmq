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

use crate::log;
use crate::metrics::ServerMetrics;
use std::{net::SocketAddr, str::FromStr};
use tokio::runtime::Runtime;

mod management_api;
mod prometheus;
mod route;
mod welcome;

#[derive(Debug)]
pub struct AdminServer<'a> {
    addr: String,
    port: Option<u16>,
    worker_threads: usize,
    server_metrics: &'a ServerMetrics,
    runtime: Runtime,
}

impl<'a> AdminServer<'a> {
    pub fn new(
        addr: String,
        port: Option<u16>,
        worker_threads: usize,
        server_metrics: &'a ServerMetrics,
    ) -> Self {
        let runtime: Runtime = tokio::runtime::Builder::new_current_thread()
            .worker_threads(worker_threads)
            .max_blocking_threads(2048)
            .thread_name("admin-http")
            .enable_io()
            .build()
            .unwrap();

        return AdminServer {
            addr,
            port,
            worker_threads,
            server_metrics,
            runtime,
        };
    }

    pub fn start(&self) {
        let _guard = self.runtime.enter();
        let ip = format!("{}:{}", self.addr, self.port.unwrap());
        
        self.runtime.spawn(async move {
            let app = route::router_construct();

            let ip_addr = SocketAddr::from_str(&ip).unwrap();
            log::info(&format!("http server start success. bind:{}", ip));

            axum::Server::bind(&ip_addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
    }

    pub fn stop(&self) {
        println!("{},{:?}", self.worker_threads, self.server_metrics)
        // self.runtime.shutdown_timeout(Duration::from_secs(30));
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
