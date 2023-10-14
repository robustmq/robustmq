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

use rlog;
use std::net::SocketAddr;

pub mod server;

pub fn new(ip: &str, port: Option<u16>) {
    server::register();
    self::start_prometheus_export(ip, port);
}

fn start_prometheus_export(ip: &str, port: Option<u16>) {
    let addr_raw = format!("{}:{}", ip, port.unwrap());
    rlog::info(&format!("prometrics export Binding address: {}", addr_raw));

    let addr: SocketAddr = addr_raw
        .parse()
        .expect(&format!("can not parse listen addr,addr:{:?}", addr_raw));
    prometheus_exporter::start(addr).expect("can not start peometheus exporter");

    rlog::info("Prometrics Export started successfully. ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        self::new("", 1201)
    }
}
