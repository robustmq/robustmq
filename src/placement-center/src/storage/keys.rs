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

/** ===========Raft========== */
pub fn key_name_by_first_index() -> String {
    "/raft/first_index".to_string()
}

pub fn key_name_by_last_index() -> String {
    "/raft/last_index".to_string()
}

pub fn key_name_by_hard_state() -> String {
    "/raft/hard_state".to_string()
}

pub fn key_name_by_conf_state() -> String {
    "/raft/conf_state".to_string()
}

pub fn key_name_by_entry(idx: u64) -> String {
    format!("/raft/entry/{}", idx)
}

pub fn key_name_uncommit(idx: u64) -> String {
    format!("/raft/uncommit_index/{}", idx)
}

pub fn key_name_uncommit_prefix() -> String {
    "/raft/uncommit_index".to_string()
}

/** ===========Cluster========== */
pub fn key_cluster(cluster_type: &str, cluster_name: &str) -> String {
    format!("/clusters/{}/{}", cluster_type, cluster_name)
}

pub fn key_cluster_prefix() -> String {
    "/clusters/".to_string()
}

pub fn key_cluster_prefix_by_type(cluster_type: &str) -> String {
    format!("/clusters/{}", cluster_type)
}

pub fn key_node(cluster_name: &str, node_id: u64) -> String {
    format!("/clusters/node/{}/{}", cluster_name, node_id)
}

pub fn key_node_prefix(cluster_name: &str) -> String {
    format!("/clusters/node/{}", cluster_name)
}

pub fn key_node_prefix_all() -> String {
    "/clusters/node/".to_string()
}

pub fn key_resource_config(cluster_name: String, resource_key: String) -> String {
    format!("/config/{}/{}", cluster_name, resource_key)
}

pub fn key_resource_idempotent(cluster_name: &str, produce_id: &str, seq_num: u64) -> String {
    format!("/idempotent/{}/{}/{}", cluster_name, produce_id, seq_num)
}

/** ===========Journal========== */
pub fn key_shard(cluster_name: &str, namespace: &str, shard_name: &str) -> String {
    format!(
        "/journal//shard/{}/{}/{}",
        cluster_name, namespace, shard_name
    )
}

#[allow(dead_code)]
pub fn key_shard_prefix(cluster_name: &str) -> String {
    format!("/journal/shard/{}", cluster_name)
}

pub fn key_segment(cluster_name: &str, shard_name: &str, segement_seq: u64) -> String {
    format!(
        "/journal/segment/{}/{}/{}",
        cluster_name, shard_name, segement_seq
    )
}

pub fn key_segment_cluster_prefix(cluster_name: &str) -> String {
    format!("/journal/segment/{}", cluster_name)
}

pub fn key_segment_shard_prefix(cluster_name: &str, shard_name: &str) -> String {
    format!("/journal/segment/{}/{}", cluster_name, shard_name)
}

/** ===========MQTT========== */
pub fn storage_key_mqtt_user(cluster_name: &str, user_name: &str) -> String {
    format!("/mqtt/user/{}/{}", cluster_name, user_name)
}

pub fn storage_key_mqtt_user_cluster_prefix(cluster_name: &str) -> String {
    format!("/mqtt/user/{}", cluster_name)
}

pub fn storage_key_mqtt_topic(cluster_name: &str, user_name: &str) -> String {
    format!("/mqtt/topic/{}/{}", cluster_name, user_name)
}

pub fn storage_key_mqtt_topic_cluster_prefix(cluster_name: &str) -> String {
    format!("/mqtt/topic/{}", cluster_name)
}

pub fn storage_key_mqtt_session(cluster_name: &str, client_id: &str) -> String {
    format!("/mqtt/session/{}/{}", cluster_name, client_id)
}

pub fn storage_key_mqtt_session_cluster_prefix(cluster_name: &str) -> String {
    format!("/mqtt/session/{}", cluster_name)
}

pub fn storage_key_mqtt_last_will(cluster_name: &str, client_id: &str) -> String {
    format!("/mqtt/lastwill/{}/{}", cluster_name, client_id)
}
pub fn storage_key_mqtt_last_will_prefix(cluster_name: &str) -> String {
    format!("/mqtt/lastwill/{}", cluster_name)
}

pub fn storage_key_mqtt_node_sub_group_leader(cluster_name: &str) -> String {
    format!("/mqtt/sub_group_leader/{}", cluster_name)
}

pub fn storage_key_mqtt_acl(
    cluster_name: &str,
    resource_type: &str,
    resource_name: &str,
) -> String {
    format!(
        "/mqtt/acl/{}/{}/{}",
        cluster_name, resource_type, resource_name
    )
}

pub fn storage_key_mqtt_acl_prefix(cluster_name: &str) -> String {
    format!("/mqtt/acl/{}", cluster_name)
}

pub fn storage_key_mqtt_blacklist(
    cluster_name: &str,
    black_list_type: &str,
    resource_name: &str,
) -> String {
    format!(
        "/mqtt/blacklist/{}/{}/{}",
        cluster_name, black_list_type, resource_name
    )
}

pub fn storage_key_mqtt_blacklist_prefix(cluster_name: &str) -> String {
    format!("/mqtt/blacklist/{}", cluster_name)
}
