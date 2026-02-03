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

use local_ip_address::list_afinet_netifas;
use std::fs;
use std::future::Future;
use std::net::IpAddr;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::select;
use tokio::sync::broadcast;
use tonic::Status;
use tracing::{debug, warn};

use crate::enum_type::time_unit_enum::TimeUnit;
use crate::error::common::CommonError;
use crate::error::ResultCommonError;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3600;
const SECONDS_PER_DAY: u64 = 86400;

const LINK_LOCAL_PREFIX: [u8; 2] = [169, 254];
const SKIP_INTERFACE_PREFIXES: &[&str] =
    &["docker", "br-", "utun", "tun", "tap", "ppp", "awdl", "llw"];

/// Create multi-level directory if it doesn't exist
pub fn try_create_fold(fold: impl AsRef<Path>) -> Result<(), CommonError> {
    let fold = fold.as_ref();
    if !fold.exists() {
        fs::create_dir_all(fold)?
    }
    Ok(())
}

/// Returns milliseconds since Unix epoch
pub fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis()
}

/// Returns nanoseconds since Unix epoch
pub fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_nanos()
}

/// Returns seconds since Unix epoch
pub fn now_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_secs()
}

pub fn convert_seconds(number: u64, unit: TimeUnit) -> u64 {
    match unit {
        TimeUnit::Seconds => number,
        TimeUnit::Minutes => number * SECONDS_PER_MINUTE,
        TimeUnit::Hours => number * SECONDS_PER_HOUR,
        TimeUnit::Days => number * SECONDS_PER_DAY,
    }
}

/// Get local IP address with priority: env var ROBUSTMQ_BROKER_IP > smart detection > 127.0.0.1
pub fn get_local_ip() -> String {
    if let Ok(ip_str) = std::env::var("ROBUSTMQ_BROKER_IP") {
        if ip_str.parse::<IpAddr>().is_ok() {
            return ip_str;
        }
        warn!("Invalid IP in ROBUSTMQ_BROKER_IP: {}", ip_str);
    }

    match get_local_ip_smart() {
        Ok(ip) => ip,
        Err(e) => {
            debug!(
                "Smart IP detection failed: {}, falling back to 127.0.0.1",
                e
            );
            "127.0.0.1".to_string()
        }
    }
}

/// Smart IP detection: prioritizes physical interfaces > private IPs > public IPs
fn get_local_ip_smart() -> Result<String, String> {
    let interfaces = list_afinet_netifas().map_err(|e| e.to_string())?;

    let mut physical_ips = Vec::new();
    let mut private_ips = Vec::new();
    let mut public_ips = Vec::new();

    for (name, ip) in interfaces {
        if should_skip_interface(&name, &ip) {
            continue;
        }

        if is_physical_interface(&name) {
            physical_ips.push((name, ip));
        } else if is_private_ip(&ip) {
            private_ips.push((name, ip));
        } else {
            public_ips.push((name, ip));
        }
    }

    if let Some(ip) = select_best_ip(&mut physical_ips) {
        return Ok(ip);
    }
    if let Some(ip) = select_best_ip(&mut private_ips) {
        return Ok(ip);
    }
    if let Some(ip) = select_best_ip(&mut public_ips) {
        return Ok(ip);
    }

    Err("No suitable network interface found".to_string())
}

#[cfg_attr(test, allow(dead_code))]
pub(crate) fn should_skip_interface(name: &str, ip: &IpAddr) -> bool {
    if ip.is_loopback() || ip.is_ipv6() {
        return true;
    }

    if SKIP_INTERFACE_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return true;
    }

    if let IpAddr::V4(ipv4) = ip {
        let octets = ipv4.octets();
        if octets[0] == LINK_LOCAL_PREFIX[0] && octets[1] == LINK_LOCAL_PREFIX[1] {
            return true;
        }
    }

    false
}

#[cfg_attr(test, allow(dead_code))]
pub(crate) fn is_physical_interface(name: &str) -> bool {
    name.starts_with("en")
        || name.starts_with("eth")
        || name.starts_with("wlan")
        || name.starts_with("ens")
        || name.starts_with("enp")
        || name.starts_with("eno")
}

#[cfg_attr(test, allow(dead_code))]
pub(crate) fn is_private_ip(ip: &IpAddr) -> bool {
    if let IpAddr::V4(ipv4) = ip {
        let octets = ipv4.octets();

        if octets[0] == 10 {
            return true;
        }

        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
            return true;
        }

        if octets[0] == 192 && octets[1] == 168 {
            return true;
        }
    }
    false
}

fn select_best_ip(candidates: &mut [(String, IpAddr)]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by_key(|(_, ip)| {
        if let IpAddr::V4(ipv4) = ip {
            let octets = ipv4.octets();

            if octets[0] == 192 && octets[1] == 168 {
                return 0;
            }

            if octets[0] == 10 {
                return 1;
            }

            if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                return 2;
            }
        }

        100
    });

    candidates.first().map(|(_, ip)| ip.to_string())
}

pub fn local_hostname() -> String {
    "127.0.0.1".to_string()
}

pub fn get_addr_by_local_hostname(port: u32) -> String {
    format!("127.0.0.1:{port}")
}

pub fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn read_file(path: impl AsRef<Path>) -> Result<String, CommonError> {
    Ok(fs::read_to_string(path)?)
}

pub fn serialize_value<T>(value: &T) -> Result<String, Status>
where
    T: serde::Serialize,
{
    serde_json::to_string(value)
        .map_err(|e| Status::cancelled(CommonError::CommonError(e.to_string()).to_string()))
}

pub async fn loop_select_ticket<F, Fut>(
    ac_fn: F,
    tick_millis: u64,
    stop_sx: &broadcast::Sender<bool>,
) where
    F: FnOnce() -> Fut + Copy,
    Fut: Future<Output = ResultCommonError>,
{
    let mut stop_recv = stop_sx.subscribe();
    let mut internal = tokio::time::interval(Duration::from_millis(tick_millis));
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
    use crate::tools::{convert_seconds, get_local_ip};

    use super::get_addr_by_local_hostname;

    #[test]
    fn get_local_ip_test() {
        let ip_string = get_local_ip();
        let parse_ip: std::net::IpAddr = ip_string.parse().unwrap();
        assert!(parse_ip.is_ipv4() || parse_ip.is_ipv6());
    }

    #[test]
    fn test_convert_seconds() {
        assert_eq!(convert_seconds(1, TimeUnit::Seconds), 1);
        assert_eq!(convert_seconds(1, TimeUnit::Minutes), 60);
        assert_eq!(convert_seconds(2, TimeUnit::Hours), 7200);
        assert_eq!(convert_seconds(1, TimeUnit::Days), 86400);

        assert_eq!(convert_seconds(0, TimeUnit::Days), 0);
        assert_eq!(convert_seconds(10, TimeUnit::Minutes), 600);
    }

    #[test]
    fn test_get_addr_by_local_hostname() {
        let ip = get_addr_by_local_hostname(1883);
        let _: SocketAddr = ip.parse().unwrap();
    }

    #[test]
    fn test_get_local_ip_with_env() {
        std::env::set_var("ROBUSTMQ_BROKER_IP", "10.0.0.1");
        let ip = get_local_ip();
        assert_eq!(ip, "10.0.0.1");
        std::env::remove_var("ROBUSTMQ_BROKER_IP");
    }

    #[test]
    fn test_is_physical_interface() {
        use crate::tools::is_physical_interface;
        assert!(is_physical_interface("en0"));
        assert!(is_physical_interface("eth0"));
        assert!(is_physical_interface("wlan0"));
        assert!(is_physical_interface("ens33"));
        assert!(!is_physical_interface("docker0"));
        assert!(!is_physical_interface("utun0"));
    }

    #[test]
    fn test_is_private_ip() {
        use crate::tools::is_private_ip;
        use std::net::IpAddr;

        assert!(is_private_ip(&"192.168.1.1".parse::<IpAddr>().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse::<IpAddr>().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse::<IpAddr>().unwrap()));
        assert!(!is_private_ip(&"8.8.8.8".parse::<IpAddr>().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn test_should_skip_interface() {
        use crate::tools::should_skip_interface;
        use std::net::IpAddr;

        let loopback: IpAddr = "127.0.0.1".parse().unwrap();
        let normal: IpAddr = "192.168.1.1".parse().unwrap();
        let link_local: IpAddr = "169.254.1.1".parse().unwrap();

        assert!(should_skip_interface("lo0", &loopback));
        assert!(should_skip_interface("docker0", &normal));
        assert!(should_skip_interface("utun0", &normal));
        assert!(should_skip_interface("awdl0", &normal));
        assert!(should_skip_interface("br-123", &normal));
        assert!(should_skip_interface("tap0", &normal));
        assert!(should_skip_interface("en0", &link_local));
        assert!(!should_skip_interface("en0", &normal));
        assert!(!should_skip_interface("eth0", &normal));
    }

    #[test]
    fn test_time_functions() {
        let millis = super::now_millis();
        let nanos = super::now_nanos();
        let secs = super::now_second();

        assert!(millis > 0);
        assert!(nanos > millis * 1_000_000);
        assert!(secs > 0);
        assert!(secs < millis as u64);
    }

    #[test]
    fn test_file_exists() {
        use super::file_exists;
        assert!(file_exists("/"));
        assert!(!file_exists("/non/existent/path/to/file.txt"));
    }

    #[test]
    fn test_get_local_ip_with_invalid_env() {
        std::env::set_var("ROBUSTMQ_BROKER_IP", "invalid_ip_address");
        let ip = get_local_ip();
        assert_ne!(ip, "invalid_ip_address");
        std::env::remove_var("ROBUSTMQ_BROKER_IP");
    }
}
