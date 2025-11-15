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

use common_base::{error::common::CommonError, utils::serialize::serialize};

pub fn key_last_purged_log_id(machine: &str) -> String {
    format!("{}/store/last_purged_log_id", machine)
}

pub fn key_committed(machine: &str) -> String {
    format!("{}/store/committed", machine)
}

pub fn key_vote(machine: &str) -> String {
    format!("{}/store/vote", machine)
}

pub fn key_raft_log(machine: &str, index: u64) -> Result<Vec<u8>, CommonError> {
    serialize(&format!("{}/log/{}", machine, index))
}

pub fn raft_log_key_to_id(_machine: &str, _key: &[u8]) -> Result<u64, CommonError> {
    // let prefix = format!("{}/log/", machine);
    // let res = key.replace(prefix, "");
    Ok(0)
}
