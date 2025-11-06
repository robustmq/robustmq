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

use std::sync::OnceLock;

use chrono::{DateTime, FixedOffset, Local};
use chrono_tz::Tz;

static TIME_ZONE: OnceLock<String> = OnceLock::new();

pub fn timestamp_to_local_datetime(timestamp: i64) -> String {
    timestamp_to_timezone_datetime(
        timestamp,
        TIME_ZONE.get_or_init(|| Local::now().offset().to_string()),
    )
    .unwrap()
}

pub fn timestamp_to_timezone_datetime(timestamp: i64, timezone: &str) -> Result<String, String> {
    let tz = parse_timezone(timezone)?;
    let date_time =
        DateTime::from_timestamp(timestamp, 0).ok_or_else(|| "Invalid timestamp".to_string())?;

    Ok(match tz {
        TimeZoneWrapper::Tz(tz) => date_time
            .with_timezone(&tz)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),

        TimeZoneWrapper::FixedOffset(offset) => date_time
            .with_timezone(&offset)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}

enum TimeZoneWrapper {
    Tz(Tz),
    FixedOffset(FixedOffset),
}

fn parse_timezone(timezone: &str) -> Result<TimeZoneWrapper, String> {
    if let Ok(tz) = timezone.parse::<Tz>() {
        return Ok(TimeZoneWrapper::Tz(tz));
    }

    if let Some(offset) = parse_utc_offset(timezone) {
        return Ok(TimeZoneWrapper::FixedOffset(offset));
    }

    Err(format!("Invalid timezone: '{timezone}'. Expected IANA timezone (e.g. 'Asia/Shanghai') or UTC offset (e.g. 'UTC+8')"))
}

fn parse_utc_offset(tz: impl AsRef<str>) -> Option<FixedOffset> {
    let tz = tz.as_ref().trim().to_uppercase();
    let s = tz.strip_prefix("UTC").unwrap_or(&tz);

    let (sign, offset_str) = s.split_at(1);
    if sign != "+" && sign != "-" {
        return None;
    }

    // 支持: HH, HHMM, HH:MM
    let (hours, minutes) = if let Some((h, m)) = offset_str.split_once(':') {
        (h, m)
    } else if offset_str.len() == 4 {
        // 例如 0800
        (&offset_str[..2], &offset_str[2..])
    } else {
        (offset_str, "0")
    };

    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;

    let total_seconds = match sign {
        "+" => hours * 3600 + minutes * 60,
        "-" => -(hours * 3600 + minutes * 60),
        _ => return None,
    };

    FixedOffset::east_opt(total_seconds)
}

#[cfg(test)]
mod tests {
    use crate::utils::time_util::{timestamp_to_local_datetime, timestamp_to_timezone_datetime};

    #[test]
    pub fn test_timestamp_to_local_datetime() {
        let timestamp = 1751359577;
        println!("{}", timestamp_to_local_datetime(timestamp));

        println!(
            "{}",
            timestamp_to_timezone_datetime(timestamp, "utc+8").unwrap()
        );

        println!(
            "{}",
            timestamp_to_timezone_datetime(timestamp, "+0800").unwrap()
        );

        println!(
            "{}",
            timestamp_to_timezone_datetime(timestamp, "-8").unwrap()
        );
        println!(
            "{}",
            timestamp_to_timezone_datetime(timestamp, "+0").unwrap()
        );

        let result = timestamp_to_timezone_datetime(timestamp, "Asia/Shanghai");
        assert!(result.is_ok());
        println!("Asia/Shanghai time: {}", result.unwrap());

        let result2 = timestamp_to_timezone_datetime(timestamp, "UTC");
        assert!(result2.is_ok());
        println!("UTC time: {}", result2.unwrap());

        let result3 = timestamp_to_timezone_datetime(timestamp, "America/New_York");
        assert!(result3.is_ok());
        println!("America/New_York time: {}", result3.unwrap());

        let result4 = timestamp_to_timezone_datetime(timestamp, "invalid_timezone");
        assert!(result4.is_err());
    }
}
