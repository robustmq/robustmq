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


use std::{net::SocketAddr, str::FromStr};
use tokio::runtime::Runtime;
use crate::log;

mod management_api;
mod prometheus;
mod welcome;
mod route;

pub fn start(addr: String, port: Option<u16>, worker_threads: usize) -> Runtime {
    let runtime: Runtime = tokio::runtime::Builder::new_current_thread()
        .worker_threads(worker_threads)
        .max_blocking_threads(2048)
        .thread_name("admin-http")
        .enable_io()
        .build()
        .unwrap();
    
    let _guard = runtime.enter();

    runtime.spawn(async move {
        let app = route::router_construct();

        let ip = format!("{}:{}", addr, port.unwrap());
        let ip_addr = SocketAddr::from_str(&ip).unwrap();

        log::info(&format!("http server start success. bind:{}", ip));

        axum::Server::bind(&ip_addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    return runtime;
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
       
    }
}
