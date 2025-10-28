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

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

pub mod base;
pub mod mqtt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricsValue {
    pub value: u64,
    pub timestamp: u64,
}

impl MetricsValue {
    pub fn new(value: u64, timestamp: u64) -> Self {
        Self { value, timestamp }
    }
}

pub fn calc_value(max_value: u64, min_value: u64, time_window: u64) -> u64 {
    if time_window == 0 {
        return 0;
    }
    let diff = (max_value - min_value) as f64;
    let window = time_window as f64;
    let result = diff / window;
    result.round() as u64
}

pub fn get_max_key_value(data: &DashMap<u64, u64>) -> u64 {
    if data.is_empty() {
        return 0;
    }
    data.iter()
        .max_by_key(|entry| *entry.key())
        .map(|entry| *entry.value())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_value() {
        assert_eq!(calc_value(100, 0, 10), 10);
        assert_eq!(calc_value(105, 0, 10), 11);
        assert_eq!(calc_value(0, 0, 10), 0);
        assert_eq!(calc_value(100, 0, 0), 0);
    }

    #[test]
    fn test_get_max_key_value() {
        let data: DashMap<u64, u64> = DashMap::new();
        data.insert(100, 10);
        data.insert(200, 20);
        assert_eq!(get_max_key_value(&data), 20);

        let empty: DashMap<u64, u64> = DashMap::new();
        assert_eq!(get_max_key_value(&empty), 0);
    }
}
