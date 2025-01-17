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

#[derive(Clone, Debug, PartialEq)]
pub struct PublishArgsRequest {
    pub topic: String,
    pub qos: i32,
    pub retained: bool,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubscribeArgsRequest {
    pub topic: String,
    pub qos: i32,
    pub username: String,
    pub password: String,
}
