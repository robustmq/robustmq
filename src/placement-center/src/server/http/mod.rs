// Copyright 2023 RobustMQ Team
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


pub mod index;
pub mod mqtt;
pub mod server;
pub mod journal;

pub(crate) fn v1_path(path: &str) -> String {
    return format!("/v1{}", path);
}

pub(crate) fn get_path(path: &str) -> String {
    return format!("{}/get", path);
}

pub(crate) fn create_path(path: &str) -> String {
    return format!("{}/create", path);
}

pub(crate) fn update_path(path: &str) -> String {
    return format!("{}/update", path);
}

pub(crate) fn delete_path(path: &str) -> String {
    return format!("{}/delete", path);
}

pub(crate) fn list_path(path: &str) -> String {
    return format!("{}/list", path);
}
