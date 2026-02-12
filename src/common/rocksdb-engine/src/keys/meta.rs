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

use super::PREFIX_META;

#[inline]
pub fn key_node(node_id: u64) -> String {
    format!("{}clusters/node/{}", PREFIX_META, node_id)
}

#[inline]
pub fn key_node_prefix() -> String {
    format!("{}clusters/node/", PREFIX_META)
}

#[inline]
pub fn key_resource_config(resource_key: &str) -> String {
    format!("{}config/{}", PREFIX_META, resource_key)
}

#[inline]
pub fn key_offset(group: &str, shard_name: &str) -> String {
    format!("{}offset/{}/{}", PREFIX_META, group, shard_name)
}

#[inline]
pub fn key_offset_by_group(group: &str) -> String {
    format!("{}offset/{}/", PREFIX_META, group)
}

#[inline]
pub fn key_shard(shard_name: &str) -> String {
    format!("{}journal/shard/{}", PREFIX_META, shard_name)
}

#[inline]
pub fn key_all_shard() -> &'static str {
    "/meta/journal/shard/"
}

#[inline]
pub fn key_segment(shard_name: &str, segment_seq: u32) -> String {
    format!(
        "{}journal/segment/{}/{}",
        PREFIX_META, shard_name, segment_seq
    )
}

#[inline]
pub fn key_all_segment() -> &'static str {
    "/meta/journal/segment/"
}

#[inline]
pub fn key_segment_shard_prefix(shard_name: &str) -> String {
    format!("{}journal/segment/{}/", PREFIX_META, shard_name)
}

#[inline]
pub fn key_segment_metadata(shard_name: &str, segment_seq: u32) -> String {
    format!(
        "{}journal/segment_meta/{}/{}",
        PREFIX_META, shard_name, segment_seq
    )
}

#[inline]
pub fn key_all_segment_metadata() -> &'static str {
    "/meta/journal/segment_meta/"
}

#[inline]
pub fn key_segment_metadata_shard_prefix(shard_name: &str) -> String {
    format!("{}journal/segment_meta/{}/", PREFIX_META, shard_name)
}

#[inline]
pub fn storage_key_mqtt_user(user_name: &str) -> String {
    format!("{}mqtt/user/{}", PREFIX_META, user_name)
}

#[inline]
pub fn storage_key_mqtt_user_prefix() -> String {
    format!("{}mqtt/user/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_topic(topic_name: &str) -> String {
    format!("{}mqtt/topic/{}", PREFIX_META, topic_name)
}

#[inline]
pub fn storage_key_mqtt_topic_cluster_prefix() -> String {
    format!("{}mqtt/topic/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_session(client_id: &str) -> String {
    format!("{}mqtt/session/{}", PREFIX_META, client_id)
}

#[inline]
pub fn storage_key_mqtt_session_prefix() -> String {
    format!("{}mqtt/session/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_last_will(client_id: &str) -> String {
    format!("{}mqtt/last_will/{}", PREFIX_META, client_id)
}

#[inline]
pub fn storage_key_mqtt_last_will_prefix() -> String {
    format!("{}mqtt/last_will/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_group_leader(group_name: &str) -> String {
    format!("{}mqtt/sub_group_leader/{}", PREFIX_META, group_name)
}

#[inline]
pub fn storage_key_mqtt_group_leader_prefix() -> String {
    format!("{}mqtt/sub_group_leader/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_subscribe(client_id: &str, path: &str) -> String {
    format!("{}mqtt/subscribe/{}/{}", PREFIX_META, client_id, path)
}

#[inline]
pub fn storage_key_mqtt_subscribe_client_id_prefix(client_id: &str) -> String {
    format!("{}mqtt/subscribe/{}/", PREFIX_META, client_id)
}

#[inline]
pub fn storage_key_mqtt_subscribe_prefix() -> String {
    format!("{}mqtt/subscribe/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_connector(connector_name: &str) -> String {
    format!("{}mqtt/connector/{}", PREFIX_META, connector_name)
}

#[inline]
pub fn storage_key_mqtt_connector_prefix() -> String {
    format!("{}mqtt/connector/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_schema(schema_name: &str) -> String {
    format!("{}mqtt/schema/{}", PREFIX_META, schema_name)
}

#[inline]
pub fn storage_key_mqtt_schema_prefix() -> String {
    format!("{}mqtt/schema/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_schema_bind(resource_name: &str, schema_name: &str) -> String {
    format!(
        "{}mqtt/schema_bind/{}/{}",
        PREFIX_META, resource_name, schema_name
    )
}

#[inline]
pub fn storage_key_mqtt_schema_bind_prefix_by_resource(resource_name: &str) -> String {
    format!("{}mqtt/schema_bind/{}/", PREFIX_META, resource_name)
}

#[inline]
pub fn storage_key_mqtt_schema_bind_prefix() -> String {
    format!("{}mqtt/schema_bind/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_acl(resource_type: &str, resource_name: &str) -> String {
    format!(
        "{}mqtt/acl/{}/{}",
        PREFIX_META, resource_type, resource_name
    )
}

#[inline]
pub fn storage_key_mqtt_acl_prefix() -> String {
    format!("{}mqtt/acl/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_blacklist(black_list_type: &str, resource_name: &str) -> String {
    format!(
        "{}mqtt/blacklist/{}/{}",
        PREFIX_META, black_list_type, resource_name
    )
}

#[inline]
pub fn storage_key_mqtt_blacklist_prefix() -> String {
    format!("{}mqtt/blacklist/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_topic_rewrite_rule(action: &str, source_topic: &str) -> String {
    format!(
        "{}mqtt/topic_rewrite_rule/{}/{}",
        PREFIX_META, action, source_topic
    )
}

#[inline]
pub fn storage_key_mqtt_topic_rewrite_rule_prefix() -> String {
    format!("{}mqtt/topic_rewrite_rule/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_auto_subscribe_rule(topic: &str) -> String {
    format!("{}mqtt/auto_subscribe_rule/{}", PREFIX_META, topic)
}

#[inline]
pub fn storage_key_mqtt_auto_subscribe_rule_prefix() -> String {
    format!("{}mqtt/auto_subscribe_rule/", PREFIX_META)
}

#[inline]
pub fn storage_key_mqtt_retain_message(topic_name: &str) -> String {
    format!("{}mqtt/retain_message/{}", PREFIX_META, topic_name)
}

#[inline]
pub fn storage_key_mqtt_retain_message_prefix() -> String {
    format!("{}mqtt/retain_message/", PREFIX_META)
}
