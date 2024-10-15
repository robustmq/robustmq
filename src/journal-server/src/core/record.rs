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

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KvRecord {
    pub key: Bytes,
    pub value: Bytes,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentRecord {
    pub offset: u64,
    pub sequence: u32,
    pub timestamp: u64,
    pub size: u32,
    pub is_compressed: bool,
    pub key_size: u32,
    pub key: Bytes,
    pub value_size: u32,
    pub value: Bytes,
}
