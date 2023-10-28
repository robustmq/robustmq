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

use crate::config::server::RobustConfig;
use crate::log;
use std::{net::SocketAddr, str::FromStr};
use tokio::runtime::Runtime;

mod common;
mod management_api;
mod route;

#[derive(Debug)]
pub struct AdminServer<'a> {
    config: &'a RobustConfig,
}

impl<'a> AdminServer<'a> {
    pub fn new(config: &'a RobustConfig) -> Self {
        return AdminServer { config };
    }

    pub fn start(&self) -> Runtime {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.admin.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("admin-http")
            .enable_io()
            .build()
            .unwrap();

        let _guard = runtime.enter();
        let ip = format!("{}:{}", self.config.addr, self.config.admin.port.unwrap());

        runtime.spawn(async move {
            let ip_addr = SocketAddr::from_str(&ip).unwrap();
    
            let app = route::routes();
    
            log::info(&format!("http server start success. bind:{}", ip));

            axum::Server::bind(&ip_addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        return runtime;
    }

    fn _print_config(&self) {
        println!("{}", self.config.addr);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
