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

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectPayload {
    client_id: String,
    will_topic: Option<String>,
    will_message: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[allow(dead_code)]
impl ConnectPayload {
    pub fn new(
        client_id: String,
        will_topic: Option<String>,
        will_message: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        ConnectPayload {
            client_id,
            will_topic,
            will_message,
            username,
            password,
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn will_topic(&self) -> Option<&String> {
        self.will_topic.as_ref()
    }

    pub fn will_message(&self) -> Option<&String> {
        self.will_message.as_ref()
    }

    pub fn username(&self) -> Option<&String> {
        self.username.as_ref()
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }
}
