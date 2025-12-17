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

mod macros;

pub mod broker;
pub mod meta;
pub mod pool;
mod utils;
// const MAX_RETRY_TIMES: usize = 10;

pub fn retry_times() -> usize {
    0
}

pub fn retry_sleep_time(times: usize) -> u64 {
    (times * 2) as u64
}
