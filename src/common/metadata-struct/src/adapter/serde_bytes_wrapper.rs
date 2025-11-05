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

//! Custom serde wrapper for bytes::Bytes to work with bincode
//!
//! bincode doesn't natively support bytes::Bytes serialization,
//! so we convert to/from Vec<u8> during serialization.

use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert Bytes to Vec<u8> for serialization
    bytes.to_vec().serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize as Vec<u8> then convert to Bytes
    let vec = Vec::<u8>::deserialize(deserializer)?;
    Ok(Bytes::from(vec))
}
