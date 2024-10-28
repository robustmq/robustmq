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

use std::net::SocketAddr;

use axum::async_trait;
use common_base::error::mqtt_broker::MqttBrokerError;

pub mod http;
pub mod jwt;
pub mod plaintext;
pub mod psk;
pub mod x509;

#[async_trait]
pub trait Authentication {
    async fn apply(&self) -> Result<bool, MqttBrokerError>;
}

pub fn is_ip_blacklist(_: &SocketAddr) -> bool {
    false
}

#[cfg(test)]
mod test {
    use super::is_ip_blacklist;

    #[tokio::test]
    pub async fn is_ip_blacklist_test() {
        assert!(!is_ip_blacklist(&"127.0.0.1:1000".parse().unwrap()))
    }
}
