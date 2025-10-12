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

/** ===========Cluster========== */
pub fn key_cluster(cluster_name: &str) -> String {
    prefix_key(format!("/clusters/{cluster_name}"))
}

pub fn key_cluster_prefix() -> String {
    prefix_key("/clusters/".to_string())
}

pub fn key_node(cluster_name: &str, node_id: u64) -> String {
    prefix_key(format!("/clusters/node/{cluster_name}/{node_id}"))
}

pub fn key_node_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/clusters/node/{cluster_name}/"))
}

pub fn key_node_prefix_all() -> String {
    prefix_key("/clusters/node/".to_string())
}

pub fn key_resource_config(cluster_name: String, resource_key: String) -> String {
    prefix_key(format!("/config/{cluster_name}/{resource_key}"))
}

pub fn key_resource_idempotent(cluster_name: &str, produce_id: &str, seq_num: u64) -> String {
    prefix_key(format!("/idempotent/{cluster_name}/{produce_id}/{seq_num}"))
}

pub fn key_offset(cluster_name: &str, group: &str, namespace: &str, shard_name: &str) -> String {
    prefix_key(format!(
        "/offset/{cluster_name}/{group}/{namespace}/{shard_name}"
    ))
}

pub fn key_offset_by_group(cluster_name: &str, group: &str) -> String {
    prefix_key(format!("/offset/{cluster_name}/{group}"))
}

/** ===========Journal========== */
pub fn key_shard(cluster_name: &str, namespace: &str, shard_name: &str) -> String {
    prefix_key(format!(
        "/journal/shard/{cluster_name}/{namespace}/{shard_name}"
    ))
}

pub fn key_shard_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/journal/shard/{cluster_name}/"))
}

pub fn key_shard_namespace_prefix(cluster_name: &str, namespace: &str) -> String {
    prefix_key(format!("/journal/shard/{cluster_name}/{namespace}/"))
}

pub fn key_all_shard() -> String {
    prefix_key("/journal/shard/".to_string())
}

pub fn key_segment(
    cluster_name: &str,
    namespace: &str,
    shard_name: &str,
    segment_seq: u32,
) -> String {
    prefix_key(format!(
        "/journal/segment/{cluster_name}/{namespace}/{shard_name}/{segment_seq}"
    ))
}

pub fn key_all_segment() -> String {
    prefix_key("/journal/segment/".to_string())
}

pub fn key_segment_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/journal/segment/{cluster_name}/"))
}

pub fn key_segment_namespace_prefix(cluster_name: &str, namespace: &str) -> String {
    prefix_key(format!("/journal/segment/{cluster_name}/{namespace}/"))
}

pub fn key_segment_shard_prefix(cluster_name: &str, namespace: &str, shard_name: &str) -> String {
    prefix_key(format!(
        "/journal/segment/{cluster_name}/{namespace}/{shard_name}/"
    ))
}

pub fn key_segment_metadata(
    cluster_name: &str,
    namespace: &str,
    shard_name: &str,
    segment_seq: u32,
) -> String {
    prefix_key(format!(
        "/journal/segmentmeta/{cluster_name}/{namespace}/{shard_name}/{segment_seq}"
    ))
}

pub fn key_all_segment_metadata() -> String {
    prefix_key("/journal/segmentmeta/".to_string())
}

pub fn key_segment_metadata_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/journal/segmentmeta/{cluster_name}/"))
}

pub fn key_segment_metadata_namespace_prefix(cluster_name: &str, namespace: &str) -> String {
    prefix_key(format!("/journal/segmentmeta/{cluster_name}/{namespace}/"))
}

pub fn key_segment_metadata_shard_prefix(
    cluster_name: &str,
    namespace: &str,
    shard_name: &str,
) -> String {
    prefix_key(format!(
        "/journal/segmentmeta/{cluster_name}/{namespace}/{shard_name}/"
    ))
}

/** ===========MQTT========== */
pub fn storage_key_mqtt_user(cluster_name: &str, user_name: &str) -> String {
    prefix_key(format!("/mqtt/user/{cluster_name}/{user_name}"))
}

pub fn storage_key_mqtt_user_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/user/{cluster_name}/"))
}

pub fn storage_key_mqtt_topic(cluster_name: &str, user_name: &str) -> String {
    prefix_key(format!("/mqtt/topic/{cluster_name}/{user_name}"))
}

pub fn storage_key_mqtt_topic_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/topic/{cluster_name}/"))
}

pub fn storage_key_mqtt_session(cluster_name: &str, client_id: &str) -> String {
    prefix_key(format!("/mqtt/session/{cluster_name}/{client_id}"))
}

pub fn storage_key_mqtt_session_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/session/{cluster_name}/"))
}

pub fn storage_key_mqtt_last_will(cluster_name: &str, client_id: &str) -> String {
    prefix_key(format!("/mqtt/lastwill/{cluster_name}/{client_id}"))
}
pub fn storage_key_mqtt_last_will_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/lastwill/{cluster_name}/"))
}

pub fn storage_key_mqtt_node_sub_group_leader(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/sub_group_leader/{cluster_name}"))
}

pub fn storage_key_mqtt_subscribe(cluster_name: &str, client_id: &str, path: &str) -> String {
    prefix_key(format!("/mqtt/subscribe/{cluster_name}/{client_id}/{path}"))
}

pub fn storage_key_mqtt_subscribe_client_id_prefix(cluster_name: &str, client_id: &str) -> String {
    prefix_key(format!("/mqtt/subscribe/{cluster_name}/{client_id}/"))
}

pub fn storage_key_mqtt_subscribe_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/subscribe/{cluster_name}/"))
}

pub fn storage_key_mqtt_connector(cluster_name: &str, connector_name: &str) -> String {
    prefix_key(format!("/mqtt/connector/{cluster_name}/{connector_name}"))
}

pub fn storage_key_mqtt_connector_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/connector/{cluster_name}"))
}

pub fn storage_key_mqtt_schema(cluster_name: &str, schema_name: &str) -> String {
    prefix_key(format!("/mqtt/schema/{cluster_name}/{schema_name}"))
}

pub fn storage_key_mqtt_schema_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/schema/{cluster_name}"))
}

pub fn storage_key_mqtt_schema_bind(
    cluster_name: &str,
    resource_name: &str,
    schema_name: &str,
) -> String {
    prefix_key(format!(
        "/mqtt/schema_bind/{cluster_name}/{resource_name}/{schema_name}"
    ))
}

pub fn storage_key_mqtt_schema_bind_prefix_by_resource(
    cluster_name: &str,
    resource_name: &str,
) -> String {
    prefix_key(format!("/mqtt/schema_bind/{cluster_name}/{resource_name}"))
}

pub fn storage_key_mqtt_schema_bind_prefix_by_cluster(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/schema_bind/{cluster_name}"))
}

pub fn storage_key_mqtt_acl(
    cluster_name: &str,
    resource_type: &str,
    resource_name: &str,
) -> String {
    prefix_key(format!(
        "/mqtt/acl/{cluster_name}/{resource_type}/{resource_name}"
    ))
}

pub fn storage_key_mqtt_acl_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/acl/{cluster_name}/"))
}

pub fn storage_key_mqtt_blacklist(
    cluster_name: &str,
    black_list_type: &str,
    resource_name: &str,
) -> String {
    prefix_key(format!(
        "/mqtt/blacklist/{cluster_name}/{black_list_type}/{resource_name}"
    ))
}

pub fn storage_key_mqtt_blacklist_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/blacklist/{cluster_name}/"))
}

pub fn storage_key_mqtt_topic_rewrite_rule(
    cluster_name: &str,
    action: &str,
    source_topic: &str,
) -> String {
    prefix_key(format!(
        "/mqtt/topic_rewrite_rule/{cluster_name}/{action}/{source_topic}"
    ))
}

pub fn storage_key_mqtt_topic_rewrite_rule_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/topic_rewrite_rule/{cluster_name}/"))
}

pub fn storage_key_mqtt_auto_subscribe_rule(cluster_name: &str, topic: &str) -> String {
    prefix_key(format!("/mqtt/auto_subscribe_rule/{cluster_name}/{topic}"))
}

pub fn storage_key_mqtt_auto_subscribe_rule_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/auto_subscribe_rule/{cluster_name}/"))
}

pub fn storage_key_mqtt_retain_message(cluster_name: &str, topic_id: &str) -> String {
    prefix_key(format!(
        "/mqtt/topic_retain_message/{cluster_name}/{topic_id}"
    ))
}

pub fn storage_key_mqtt_retain_message_cluster_prefix(cluster_name: &str) -> String {
    prefix_key(format!("/mqtt/topic_retain_message/{cluster_name}"))
}

fn prefix_key(key: String) -> String {
    format!("/meta/{key}")
}
