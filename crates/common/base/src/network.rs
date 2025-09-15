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

use std::{net::TcpStream, time::Duration};

pub fn is_port_open(address: &str) -> bool {
    let timeout = Duration::from_secs(1);
    TcpStream::connect_timeout(&address.parse().unwrap(), timeout).is_ok()
}

pub fn broker_not_available(err: &str) -> bool {
    err.contains("Broken pip")
}
