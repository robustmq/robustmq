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

use common_base::tools::now_second;

/// Represents the state of an established NATS client connection.
/// Fields are populated from the CONNECT packet sent by the client.
#[derive(Clone, Debug)]
pub struct NatsConnection {
    // Connection ID assigned by the server
    pub connect_id: u64,

    // The IP address of the client
    pub source_ip_addr: String,

    // Whether to send +OK for every command (from CONNECT verbose field)
    pub verbose: bool,

    // Whether to enable strict subject/format validation (from CONNECT pedantic field)
    pub pedantic: bool,

    // Whether the client requires TLS (from CONNECT tls_required field)
    pub tls_required: bool,

    // Whether to echo messages published by this client back to its own subscriptions
    pub echo: bool,

    // Whether the client supports message Headers (HPUB/HMSG)
    pub headers: bool,

    // Whether the client wants immediate -ERR when publishing to a subject with no subscribers
    pub no_responders: bool,

    // Client name for logging/debugging
    pub client_name: String,

    // Client language (e.g. "go", "rust", "java")
    pub lang: String,

    // Client library version
    pub version: String,

    // Mark whether the connection has completed authentication
    pub is_login: bool,

    // The authenticated username (set after successful auth)
    pub login_user: Option<String>,

    // Time when the connection was created
    pub create_time: u64,
}

impl NatsConnection {
    pub fn new(connect_id: u64, source_ip_addr: String) -> Self {
        NatsConnection {
            connect_id,
            source_ip_addr,
            verbose: false,
            pedantic: false,
            tls_required: false,
            echo: true,
            headers: true,
            no_responders: false,
            client_name: String::new(),
            lang: String::new(),
            version: String::new(),
            is_login: false,
            login_user: None,
            create_time: now_second(),
        }
    }

    pub fn login_success(&mut self, user_name: String) {
        self.is_login = true;
        self.login_user = Some(user_name);
    }
}
