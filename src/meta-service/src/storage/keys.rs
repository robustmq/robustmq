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
#[inline]
pub fn key_node(node_id: u64) -> String {
    format!("/meta/clusters/node/{node_id}")
}

#[inline]
pub fn key_node_prefix() -> String {
    "/meta/clusters/node/".to_string()
}

#[inline]
pub fn key_resource_config(resource_key: &str) -> String {
    format!("/meta/config/{resource_key}")
}

#[inline]
pub fn key_offset(group: &str, namespace: &str, shard_name: &str) -> String {
    format!("/meta/offset/{group}/{namespace}/{shard_name}")
}

#[inline]
pub fn key_offset_by_group(group: &str) -> String {
    format!("/meta/offset/{group}")
}

/** ===========Journal========== */
#[inline]
pub fn key_shard(namespace: &str, shard_name: &str) -> String {
    format!("/meta/journal/shard/{namespace}/{shard_name}")
}

#[inline]
pub fn key_shard_namespace_prefix(namespace: &str) -> String {
    format!("/meta/journal/shard/{namespace}/")
}

#[inline]
pub fn key_all_shard() -> &'static str {
    "/meta/journal/shard/"
}

#[inline]
pub fn key_segment(namespace: &str, shard_name: &str, segment_seq: u32) -> String {
    format!("/meta/journal/segment/{namespace}/{shard_name}/{segment_seq}")
}

#[inline]
pub fn key_all_segment() -> &'static str {
    "/meta/journal/segment/"
}

#[inline]
pub fn key_segment_namespace_prefix(namespace: &str) -> String {
    format!("/meta/journal/segment/{namespace}/")
}

#[inline]
pub fn key_segment_shard_prefix(namespace: &str, shard_name: &str) -> String {
    format!("/meta/journal/segment/{namespace}/{shard_name}/")
}

#[inline]
pub fn key_segment_metadata(namespace: &str, shard_name: &str, segment_seq: u32) -> String {
    format!("/meta/journal/segmentmeta/{namespace}/{shard_name}/{segment_seq}")
}

#[inline]
pub fn key_all_segment_metadata() -> &'static str {
    "/meta/journal/segmentmeta/"
}

#[inline]
pub fn key_segment_metadata_namespace_prefix(namespace: &str) -> String {
    format!("/meta/journal/segmentmeta/{namespace}/")
}

#[inline]
pub fn key_segment_metadata_shard_prefix(namespace: &str, shard_name: &str) -> String {
    format!("/meta/journal/segmentmeta/{namespace}/{shard_name}/")
}

/** ===========MQTT========== */
#[inline]
pub fn storage_key_mqtt_user(user_name: &str) -> String {
    format!("/meta/mqtt/user/{user_name}")
}

#[inline]
pub fn storage_key_mqtt_user_prefix() -> String {
    "/meta/mqtt/user/".to_string()
}

#[inline]
pub fn storage_key_mqtt_topic(topic_name: &str) -> String {
    format!("/meta/mqtt/topic/{topic_name}")
}

#[inline]
pub fn storage_key_mqtt_topic_cluster_prefix() -> String {
    "/meta/mqtt/topic/".to_string()
}

#[inline]
pub fn storage_key_mqtt_session(client_id: &str) -> String {
    format!("/meta/mqtt/session/{client_id}")
}

#[inline]
pub fn storage_key_mqtt_session_prefix() -> String {
    "/meta/mqtt/session/".to_string()
}

#[inline]
pub fn storage_key_mqtt_last_will(client_id: &str) -> String {
    format!("/meta/mqtt/lastwill/{client_id}")
}

#[inline]
pub fn storage_key_mqtt_last_will_prefix() -> String {
    "/meta/mqtt/lastwill/".to_string()
}

#[inline]
pub fn storage_key_mqtt_node_sub_group_leader() -> String {
    "/meta/mqtt/sub_group_leader".to_string()
}

#[inline]
pub fn storage_key_mqtt_subscribe(client_id: &str, path: &str) -> String {
    format!("/meta/mqtt/subscribe/{client_id}/{path}")
}

#[inline]
pub fn storage_key_mqtt_subscribe_client_id_prefix(client_id: &str) -> String {
    format!("/meta/mqtt/subscribe/{client_id}/")
}

#[inline]
pub fn storage_key_mqtt_subscribe_prefix() -> String {
    "/meta/mqtt/subscribe/".to_string()
}

#[inline]
pub fn storage_key_mqtt_connector(connector_name: &str) -> String {
    format!("/meta/mqtt/connector/{connector_name}")
}

#[inline]
pub fn storage_key_mqtt_connector_prefix() -> String {
    "/meta/mqtt/connector/".to_string()
}

#[inline]
pub fn storage_key_mqtt_schema(schema_name: &str) -> String {
    format!("/meta/mqtt/schema/{schema_name}")
}

#[inline]
pub fn storage_key_mqtt_schema_prefix() -> String {
    "/meta/mqtt/schema/".to_string()
}

#[inline]
pub fn storage_key_mqtt_schema_bind(resource_name: &str, schema_name: &str) -> String {
    format!("/meta/mqtt/schema_bind/{resource_name}/{schema_name}")
}

#[inline]
pub fn storage_key_mqtt_schema_bind_prefix_by_resource(resource_name: &str) -> String {
    format!("/meta/mqtt/schema_bind/{resource_name}/")
}

#[inline]
pub fn storage_key_mqtt_schema_bind_prefix() -> String {
    "/meta/mqtt/schema_bind/".to_string()
}

#[inline]
pub fn storage_key_mqtt_acl(resource_type: &str, resource_name: &str) -> String {
    format!("/meta/mqtt/acl/{resource_type}/{resource_name}")
}

#[inline]
pub fn storage_key_mqtt_acl_prefix() -> String {
    "/meta/mqtt/acl/".to_string()
}

#[inline]
pub fn storage_key_mqtt_blacklist(black_list_type: &str, resource_name: &str) -> String {
    format!("/meta/mqtt/blacklist/{black_list_type}/{resource_name}")
}

#[inline]
pub fn storage_key_mqtt_blacklist_prefix() -> String {
    "/meta/mqtt/blacklist/".to_string()
}

#[inline]
pub fn storage_key_mqtt_topic_rewrite_rule(action: &str, source_topic: &str) -> String {
    format!("/meta/mqtt/topic_rewrite_rule/{action}/{source_topic}")
}

#[inline]
pub fn storage_key_mqtt_topic_rewrite_rule_prefix() -> String {
    "/meta/mqtt/topic_rewrite_rule/".to_string()
}

#[inline]
pub fn storage_key_mqtt_auto_subscribe_rule(topic: &str) -> String {
    format!("/meta/mqtt/auto_subscribe_rule/{topic}")
}

#[inline]
pub fn storage_key_mqtt_auto_subscribe_rule_prefix() -> String {
    "/meta/mqtt/auto_subscribe_rule/".to_string()
}

#[inline]
pub fn storage_key_mqtt_retain_message(topic_name: &str) -> String {
    format!("/meta/mqtt/retain_message/{topic_name}")
}

#[inline]
pub fn storage_key_mqtt_retain_message_prefix() -> String {
    "/meta/mqtt/retain_message/".to_string()
}
