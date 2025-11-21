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

use local_ip_address::local_ip;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::select;
use tokio::sync::broadcast;
use tonic::Status;
use tracing::debug;
use uuid::Uuid;

use crate::enum_type::time_unit_enum::TimeUnit;
use crate::error::common::CommonError;
use crate::error::ResultCommonError;

/// Create multi-level directory
///
/// If the specified directory path does not exist, attempt to create the path and all its parent directories.
/// This function does not perform any operations on existing directories.
///
/// # Parameters
/// Fold - a string that specifies the directory path to be created.
///
/// # Return value
/// * `Ok(())` - Indicates that the directory has been successfully created or the specified directory already exists.
/// * `Err(CommonError)` - Indicates that the directory creation failed, and CommonError is the error type that includes the reason for the failure.
///
/// # Possible errors
/// If the directory creation fails, a 'CommonError' containing the reason for the error will be returned.
pub fn try_create_fold(fold: impl AsRef<Path>) -> Result<(), CommonError> {
    let fold = fold.as_ref();
    if !fold.exists() {
        fs::create_dir_all(fold)?
    }
    Ok(())
}

/// Retrieve the current timestamp (in milliseconds)
///
/// Returns the number of milliseconds since the Unix era (January 1, 1970 00:00:00 UTC).
/// This function uses system time and converts it to a duration with millisecond precision.
///
/// # Return value
/// Returns the number of milliseconds since the Unix era (type u128).
pub fn now_mills() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// Retrieve the current timestamp (in milliseconds)
///
/// Returns the number of milliseconds since the Unix era (January 1, 1970 00:00:00 UTC).
/// This function uses system time and converts it to a duration with millisecond precision.
///
/// # Return value
/// Returns the number of milliseconds since the Unix era (type u128).
pub fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

/// Get current seconds
///
/// # Return value
/// Returns the number of seconds since the Unix era (January 1, 1970 00:00:00 UTC).
pub fn now_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Generate a unique Uuid
///
/// This function generates a version 4 UUID (Universally Unique Identifier) and converts it to string form,
/// Remove the connecting character (-) from it
///
/// # Return value
/// String - A Uuid without connecting characters
pub fn unique_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string().replace("-", "")
}

pub fn convert_seconds(number: u64, unit: TimeUnit) -> u64 {
    if unit == TimeUnit::Minutes {
        return number * 60;
    } else if unit == TimeUnit::Hours {
        return number * 3600;
    } else if unit == TimeUnit::Days {
        return number * 86400;
    }
    number
}

/// Obtain local IP address
///
/// This function attempts to obtain the local IP address of the device and returns the address as a string upon success
/// If obtaining the IP address fails, this function will output a warning message and return the string '127.0.0.1'
///
/// # Return value
/// - When successfully obtaining a local IP address, return a string representation of the address
/// - When obtaining the IP address fails, return the string '127.0.0.1'
pub fn get_local_ip() -> String {
    match local_ip() {
        Ok(data) => data.to_string(),
        Err(e) => {
            debug!(
                "If the local IP fails, stop the process.error message:{}",
                e.to_string()
            );
            local_hostname()
        }
    }
}

pub fn local_hostname() -> String {
    "127.0.0.1".to_string()
}

pub fn get_addr_by_local_hostname(port: u32) -> String {
    format!("127.0.0.1:{port}")
}

/// Check if the file exists
///
/// # Parameters
/// * ` path ` - The path of the file, as a string reference
///
/// # Return value
/// Return a Boolean value indicating whether the file exists
pub fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Read the content of a text file
///
/// # Parameters
/// * ` path ` - a string type reference
///
/// # Return value
/// Return a result type that includes a string containing the file content upon success and a generic error upon failure
///
/// # Error
/// If the file does not exist, return a generic error indicating that the file does not exist
pub fn read_file(path: impl AsRef<Path>) -> Result<String, CommonError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(CommonError::CommonError(format!(
            "File {path:?} does not exist"
        )));
    }

    Ok(fs::read_to_string(path)?)
}

/// Serialize a DashMap object into a JSON string
/// # parameters
/// * ` value ` - a reference to a DashMap object that implements the Serialize trait
///
/// # Return value
/// Returns a result type that includes a string containing the serialized JSON data upon success and a Status object containing an error message upon failure
///
/// # Error
/// If the serialization fails, return a Status object containing an error message
pub fn serialize_value<T>(value: &T) -> Result<String, Status>
where
    T: serde::Serialize,
{
    serde_json::to_string(value)
        .map_err(|e| Status::cancelled(CommonError::CommonError(e.to_string()).to_string()))
}

pub async fn loop_select_ticket<F, Fut>(ac_fn: F, tick_secs: u64, stop_sx: &broadcast::Sender<bool>)
where
    F: FnOnce() -> Fut + Copy,
    Fut: Future<Output = ResultCommonError>,
{
    let mut stop_recv = stop_sx.subscribe();
    let mut internal = tokio::time::interval(Duration::from_secs(tick_secs));
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        break;
                    }
                }
            }
            _ = internal.tick() => {
                let _ = ac_fn().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use crate::enum_type::time_unit_enum::TimeUnit;
    use crate::tools::{convert_seconds, get_local_ip, unique_id};

    use super::get_addr_by_local_hostname;

    #[test]
    fn get_local_ip_test() {
        let ip_string = get_local_ip();
        println!("{}", ip_string);
        let parse_ip: std::net::IpAddr = ip_string.parse().unwrap();
        assert!(parse_ip.is_ipv4() || parse_ip.is_ipv6());
    }

    #[test]
    fn unique_id_test() {
        println!("{}", unique_id());
    }

    #[test]
    fn test_convert_seconds() {
        assert_eq!(convert_seconds(1, TimeUnit::Minutes), 60);
    }

    #[test]
    fn test_get_addr_by_local_hostname() {
        let ip = get_addr_by_local_hostname(1883);
        let _: SocketAddr = ip.parse().unwrap();
    }
}
