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

pub const ROLE_BROKER: &str = "broker";
pub const ROLE_META: &str = "meta";
pub const ROLE_ENGINE: &str = "engine";

pub fn is_meta_node(roles: &[String]) -> bool {
    roles.contains(&ROLE_META.to_string())
}

pub fn is_engine_node(roles: &[String]) -> bool {
    roles.contains(&ROLE_ENGINE.to_string())
}

pub fn is_broker_node(roles: &[String]) -> bool {
    roles.contains(&ROLE_BROKER.to_string())
}
