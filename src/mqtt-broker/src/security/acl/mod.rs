use crate::handler::{cache::CacheManager, connection::Connection};
use common_base::{error::common::CommonError, tools::now_second};
use ipnet::IpNet;
use metadata_struct::acl::mqtt_acl::MQTTAclAction;
use regex::Regex;
use std::str::FromStr;
use std::{net::IpAddr, sync::Arc};

pub mod metadata;

pub fn check_resource_acl(
    cache_mamanger: &Arc<CacheManager>,
    connection: &Connection,
    topic_name: &String,
    action: MQTTAclAction,
) -> Result<bool, CommonError> {
    // check super user
    if check_super_user(cache_mamanger, &connection.login_user) {
        return Ok(true);
    }

    // check blacklist
    if check_black_list(cache_mamanger, &connection) {
        return Ok(true);
    }

    // check acl
    if check_acl(cache_mamanger, &connection, &topic_name, action) {
        return Ok(true);
    }
    return Ok(true);
}

fn check_super_user(cache_mamanger: &Arc<CacheManager>, username: &String) -> bool {
    if username.is_empty() {
        return false;
    }
    if let Some(user) = cache_mamanger.user_info.get(username) {
        return user.is_superuser;
    }
    return false;
}

fn check_black_list(cache_mamanger: &Arc<CacheManager>, connection: &Connection) -> bool {
    // check user blacklist
    if let Some(data) = cache_mamanger
        .acl_metadata
        .blacklist_user
        .get(&connection.login_user)
    {
        if data.end_time < now_second() {
            return true;
        }
    }

    match cache_mamanger.acl_metadata.blacklist_user_match.read() {
        Ok(data) => {
            for raw in data.clone() {
                let re = Regex::new(&format!("^{}$", raw.resource_name)).unwrap();
                if re.is_match(&connection.login_user) {
                    if raw.end_time < now_second() {
                        return true;
                    }
                }
            }
        }
        Err(_) => {
            return false;
        }
    }

    // check client_id blacklist
    if let Some(data) = cache_mamanger
        .acl_metadata
        .blacklist_client_id
        .get(&connection.client_id)
    {
        if data.end_time < now_second() {
            return true;
        }
    }

    match cache_mamanger.acl_metadata.blacklist_client_id_match.read() {
        Ok(data) => {
            for raw in data.clone() {
                let re = Regex::new(&format!("^{}$", raw.resource_name)).unwrap();
                if re.is_match(&connection.client_id) {
                    if raw.end_time < now_second() {
                        return true;
                    }
                }
            }
        }
        Err(_) => {
            return false;
        }
    }

    // check ip blacklist
    if let Some(data) = cache_mamanger
        .acl_metadata
        .blacklist_ip
        .get(&connection.source_ip_addr)
    {
        if data.end_time < now_second() {
            return true;
        }
    }

    match cache_mamanger.acl_metadata.blacklist_user_match.read() {
        Ok(data) => {
            for raw in data.clone() {
                let ip = connection.source_ip_addr.parse::<IpAddr>().unwrap();
                let ip_cidr = IpNet::from_str(&raw.resource_name).unwrap();
                if ip_cidr.contains(&ip) {
                    if raw.end_time < now_second() {
                        return true;
                    }
                }
            }
        }
        Err(_) => {
            return false;
        }
    }

    return true;
}

fn check_acl(
    cache_mamanger: &Arc<CacheManager>,
    connection: &Connection,
    topic_name: &String,
    action: MQTTAclAction,
) -> bool {
    // check user acl

    // check client acl
    
    return true;
}
