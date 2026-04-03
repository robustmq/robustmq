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

#![allow(clippy::result_large_err)]

use crate::{auth::common::ip_match, manager::SecurityManager};
use common_base::{error::common::CommonError, tools::now_second};
use regex::Regex;
use std::sync::Arc;
use tracing::{info, warn};

pub fn is_connection_blacklisted(
    security_manager: &Arc<SecurityManager>,
    client_id: &str,
    source_ip_addr: &str,
    user: &str,
) -> Result<bool, CommonError> {
    let now = now_second();

    Ok(is_user_blacklisted(security_manager, user, now)
        || is_client_id_blacklisted(security_manager, client_id, now)
        || is_ip_blacklisted(security_manager, source_ip_addr, now))
}

fn extract_ip_from_addr(addr: &str) -> String {
    use std::net::SocketAddr;

    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return socket_addr.ip().to_string();
    }

    if let Some((ip, _port)) = addr.rsplit_once(':') {
        if ip.split('.').count() == 4 {
            return ip.to_string();
        }
    }

    warn!(source_addr = %addr, "Failed to extract IP from source address");
    addr.to_string()
}

fn wildcard_to_regex(pattern: &str) -> String {
    let mut regex_pattern = String::with_capacity(pattern.len() * 2);

    for ch in pattern.chars() {
        match ch {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                regex_pattern.push('\\');
                regex_pattern.push(ch);
            }
            _ => regex_pattern.push(ch),
        }
    }

    regex_pattern
}

fn is_active(end_time: u64, now: u64) -> bool {
    end_time > now
}

fn is_wildcard_pattern_match(target: &str, pattern: &str) -> bool {
    let regex_pattern = format!("^{}$", wildcard_to_regex(pattern));
    match Regex::new(&regex_pattern) {
        Ok(re) => re.is_match(target),
        Err(e) => {
            warn!(pattern = %pattern, error = %e, "Invalid wildcard pattern");
            false
        }
    }
}

fn is_user_blacklisted(security_manager: &Arc<SecurityManager>, user: &str, now: u64) -> bool {
    // Check exact match across all tenants
    for tenant_entry in security_manager.security_metadata.blacklist_user.iter() {
        if let Some(data) = tenant_entry.value().get(user) {
            if is_active(data.end_time, now) {
                info!(
                    username = %user,
                    end_time = data.end_time,
                    now = now,
                    "Connection blocked by exact user blacklist"
                );
                return true;
            }
        }
    }

    // Check wildcard match across all tenants
    for raw in security_manager
        .security_metadata
        .get_blacklist_user_match()
    {
        if !is_active(raw.end_time, now) {
            continue;
        }
        if is_wildcard_pattern_match(user, &raw.resource_name) {
            info!(
                username = %user,
                pattern = %raw.resource_name,
                end_time = raw.end_time,
                now = now,
                "Connection blocked by user wildcard blacklist"
            );
            return true;
        }
    }

    false
}

fn is_client_id_blacklisted(
    security_manager: &Arc<SecurityManager>,
    client_id: &str,
    now: u64,
) -> bool {
    // Check exact match across all tenants
    for tenant_entry in security_manager
        .security_metadata
        .blacklist_client_id
        .iter()
    {
        if let Some(data) = tenant_entry.value().get(client_id) {
            if is_active(data.end_time, now) {
                info!(
                    client_id = %client_id,
                    end_time = data.end_time,
                    now = now,
                    "Connection blocked by exact client_id blacklist"
                );
                return true;
            }
        }
    }

    // Check wildcard match across all tenants
    for raw in security_manager
        .security_metadata
        .get_blacklist_client_id_match()
    {
        if !is_active(raw.end_time, now) {
            continue;
        }
        if is_wildcard_pattern_match(client_id, &raw.resource_name) {
            info!(
                client_id = %client_id,
                pattern = %raw.resource_name,
                end_time = raw.end_time,
                now = now,
                "Connection blocked by client_id wildcard blacklist"
            );
            return true;
        }
    }

    false
}

fn is_ip_blacklisted(
    security_manager: &Arc<SecurityManager>,
    source_ip_addr: &str,
    now: u64,
) -> bool {
    let source_ip = extract_ip_from_addr(source_ip_addr);

    // Check exact match across all tenants
    for tenant_entry in security_manager.security_metadata.blacklist_ip.iter() {
        if let Some(data) = tenant_entry.value().get(&source_ip) {
            if is_active(data.end_time, now) {
                info!(
                    source_ip_addr = %source_ip_addr,
                    source_ip = %source_ip,
                    end_time = data.end_time,
                    now = now,
                    "Connection blocked by exact IP blacklist"
                );
                return true;
            }
        }
    }

    // Check CIDR/wildcard match across all tenants
    for raw in security_manager.security_metadata.get_blacklist_ip_match() {
        if !is_active(raw.end_time, now) {
            continue;
        }
        if ip_match(&source_ip, &raw.resource_name) {
            info!(
                source_ip_addr = %source_ip_addr,
                source_ip = %source_ip,
                pattern = %raw.resource_name,
                end_time = raw.end_time,
                now = now,
                "Connection blocked by IP pattern blacklist"
            );
            return true;
        }
    }

    false
}
