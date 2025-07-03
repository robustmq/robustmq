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

use chrono::{DateTime, Local};

pub fn timestamp_to_local_datetime(timestamp: i64) -> String {
    let date_time = DateTime::from_timestamp(timestamp, 0).unwrap();
    let local_date_time = date_time.with_timezone(&Local);
    local_date_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use crate::utils::time_util::timestamp_to_local_datetime;

    #[test]
    pub fn test_timestamp_to_local_datetime() {
        let timestamp = 1751359577; // Example timestamp
        let formatted_date = timestamp_to_local_datetime(timestamp);
        println!("Formatted date: {}", formatted_date);
        assert_eq!(formatted_date, "2025-07-01 16:46:17");
    }
}
