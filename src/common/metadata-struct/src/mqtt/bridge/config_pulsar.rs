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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PulsarConnectorConfig {
    pub server: String,
    pub topic: String,
    pub token: Option<String>,
    pub oauth: Option<String>,
    pub basic_name: Option<String>,
    pub basic_password: Option<String>,
}

impl PulsarConnectorConfig {
    pub fn new(server: String, topic: String) -> Self {
        PulsarConnectorConfig {
            server,
            topic,
            ..Default::default()
        }
    }

    pub fn with_token(&mut self, token: String) -> &mut Self {
        self.token = Some(token);
        self
    }

    pub fn with_oauth(&mut self, oauth: String) -> &mut Self {
        self.oauth = Some(oauth);
        self
    }

    pub fn with_basic(&mut self, name: String, password: String) -> &mut Self {
        self.basic_name = Some(name);
        self.basic_password = Some(password);
        self
    }
}
