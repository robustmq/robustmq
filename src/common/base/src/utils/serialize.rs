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

use crate::error::common::CommonError;

/// Serialize data to bytes using bincode
///
/// # Arguments
/// * `data` - The data to serialize
///
/// # Returns
/// * `Ok(Vec<u8>)` - Serialized bytes
/// * `Err(CommonError)` - Serialization error
pub fn serialize<T>(data: &T) -> Result<Vec<u8>, CommonError>
where
    T: Serialize,
{
    bincode::serialize(data)
        .map_err(|e| CommonError::CommonError(format!("Failed to serialize: {}", e)))
}

/// Deserialize bytes to data using bincode
///
/// # Arguments
/// * `bytes` - The bytes to deserialize
///
/// # Returns
/// * `Ok(T)` - Deserialized data
/// * `Err(CommonError)` - Deserialization error
pub fn deserialize<T>(bytes: &[u8]) -> Result<T, CommonError>
where
    T: for<'de> Deserialize<'de>,
{
    bincode::deserialize(bytes)
        .map_err(|e| CommonError::CommonError(format!("Failed to deserialize: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u64,
        name: String,
        values: Vec<i32>,
    }

    #[test]
    fn test_serialize_deserialize() {
        let data = TestData {
            id: 123,
            name: "test".to_string(),
            values: vec![1, 2, 3, 4, 5],
        };

        let serialized = serialize(&data).unwrap();
        let deserialized: TestData = deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let invalid_bytes = vec![0xFF, 0xFF, 0xFF];
        let result: Result<TestData, CommonError> = deserialize(&invalid_bytes);
        assert!(result.is_err());
    }
}
