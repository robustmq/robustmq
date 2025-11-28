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
pub fn key_cluster(cluster_name: &str) -> String {
    format!("/meta/clusters/{cluster_name}")
}

#[inline]
pub fn key_cluster_prefix() -> &'static str {
    "/meta/clusters/"
}

#[inline]
pub fn key_node(cluster_name: &str, node_id: u64) -> String {
    format!("/meta/clusters/node/{cluster_name}/{node_id}")
}

#[inline]
pub fn key_node_prefix(cluster_name: &str) -> String {
    format!("/meta/clusters/node/{cluster_name}/")
}

#[inline]
pub fn key_node_prefix_all() -> &'static str {
    "/meta/clusters/node/"
}

#[inline]
pub fn key_resource_config(cluster_name: &str, resource_key: &str) -> String {
    format!("/meta/config/{cluster_name}/{resource_key}")
}

#[inline]
pub fn key_offset(cluster_name: &str, group: &str, namespace: &str, shard_name: &str) -> String {
    format!("/meta/offset/{cluster_name}/{group}/{namespace}/{shard_name}")
}

#[inline]
pub fn key_offset_by_group(cluster_name: &str, group: &str) -> String {
    format!("/meta/offset/{cluster_name}/{group}")
}

/** ===========Journal========== */
#[inline]
pub fn key_shard(cluster_name: &str, namespace: &str, shard_name: &str) -> String {
    format!("/meta/journal/shard/{cluster_name}/{namespace}/{shard_name}")
}

#[inline]
pub fn key_shard_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/journal/shard/{cluster_name}/")
}

#[inline]
pub fn key_shard_namespace_prefix(cluster_name: &str, namespace: &str) -> String {
    format!("/meta/journal/shard/{cluster_name}/{namespace}/")
}

#[inline]
pub fn key_all_shard() -> &'static str {
    "/meta/journal/shard/"
}

#[inline]
pub fn key_segment(
    cluster_name: &str,
    namespace: &str,
    shard_name: &str,
    segment_seq: u32,
) -> String {
    format!("/meta/journal/segment/{cluster_name}/{namespace}/{shard_name}/{segment_seq}")
}

#[inline]
pub fn key_all_segment() -> &'static str {
    "/meta/journal/segment/"
}

#[inline]
pub fn key_segment_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/journal/segment/{cluster_name}/")
}

#[inline]
pub fn key_segment_namespace_prefix(cluster_name: &str, namespace: &str) -> String {
    format!("/meta/journal/segment/{cluster_name}/{namespace}/")
}

#[inline]
pub fn key_segment_shard_prefix(cluster_name: &str, namespace: &str, shard_name: &str) -> String {
    format!("/meta/journal/segment/{cluster_name}/{namespace}/{shard_name}/")
}

#[inline]
pub fn key_segment_metadata(
    cluster_name: &str,
    namespace: &str,
    shard_name: &str,
    segment_seq: u32,
) -> String {
    format!("/meta/journal/segmentmeta/{cluster_name}/{namespace}/{shard_name}/{segment_seq}")
}

#[inline]
pub fn key_all_segment_metadata() -> &'static str {
    "/meta/journal/segmentmeta/"
}

#[inline]
pub fn key_segment_metadata_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/journal/segmentmeta/{cluster_name}/")
}

#[inline]
pub fn key_segment_metadata_namespace_prefix(cluster_name: &str, namespace: &str) -> String {
    format!("/meta/journal/segmentmeta/{cluster_name}/{namespace}/")
}

#[inline]
pub fn key_segment_metadata_shard_prefix(
    cluster_name: &str,
    namespace: &str,
    shard_name: &str,
) -> String {
    format!("/meta/journal/segmentmeta/{cluster_name}/{namespace}/{shard_name}/")
}

/** ===========MQTT========== */
#[inline]
pub fn storage_key_mqtt_user(cluster_name: &str, user_name: &str) -> String {
    format!("/meta/mqtt/user/{cluster_name}/{user_name}")
}

#[inline]
pub fn storage_key_mqtt_user_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/user/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_topic(topic_name: &str) -> String {
    format!("/meta/mqtt/topic/{topic_name}")
}

#[inline]
pub fn storage_key_mqtt_topic_cluster_prefix() -> String {
    format!("/meta/mqtt/topic/")
}

#[inline]
pub fn storage_key_mqtt_session(cluster_name: &str, client_id: &str) -> String {
    format!("/meta/mqtt/session/{cluster_name}/{client_id}")
}

#[inline]
pub fn storage_key_mqtt_session_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/session/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_last_will(cluster_name: &str, client_id: &str) -> String {
    format!("/meta/mqtt/lastwill/{cluster_name}/{client_id}")
}

#[inline]
pub fn storage_key_mqtt_last_will_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/lastwill/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_node_sub_group_leader(cluster_name: &str) -> String {
    format!("/meta/mqtt/sub_group_leader/{cluster_name}")
}

#[inline]
pub fn storage_key_mqtt_subscribe(cluster_name: &str, client_id: &str, path: &str) -> String {
    format!("/meta/mqtt/subscribe/{cluster_name}/{client_id}/{path}")
}

#[inline]
pub fn storage_key_mqtt_subscribe_client_id_prefix(cluster_name: &str, client_id: &str) -> String {
    format!("/meta/mqtt/subscribe/{cluster_name}/{client_id}/")
}

#[inline]
pub fn storage_key_mqtt_subscribe_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/subscribe/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_connector(cluster_name: &str, connector_name: &str) -> String {
    format!("/meta/mqtt/connector/{cluster_name}/{connector_name}")
}

#[inline]
pub fn storage_key_mqtt_connector_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/connector/{cluster_name}")
}

#[inline]
pub fn storage_key_mqtt_schema(cluster_name: &str, schema_name: &str) -> String {
    format!("/meta/mqtt/schema/{cluster_name}/{schema_name}")
}

#[inline]
pub fn storage_key_mqtt_schema_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/schema/{cluster_name}")
}

#[inline]
pub fn storage_key_mqtt_schema_bind(
    cluster_name: &str,
    resource_name: &str,
    schema_name: &str,
) -> String {
    format!("/meta/mqtt/schema_bind/{cluster_name}/{resource_name}/{schema_name}")
}

#[inline]
pub fn storage_key_mqtt_schema_bind_prefix_by_resource(
    cluster_name: &str,
    resource_name: &str,
) -> String {
    format!("/meta/mqtt/schema_bind/{cluster_name}/{resource_name}")
}

#[inline]
pub fn storage_key_mqtt_schema_bind_prefix_by_cluster(cluster_name: &str) -> String {
    format!("/meta/mqtt/schema_bind/{cluster_name}")
}

#[inline]
pub fn storage_key_mqtt_acl(
    cluster_name: &str,
    resource_type: &str,
    resource_name: &str,
) -> String {
    format!("/meta/mqtt/acl/{cluster_name}/{resource_type}/{resource_name}")
}

#[inline]
pub fn storage_key_mqtt_acl_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/acl/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_blacklist(
    cluster_name: &str,
    black_list_type: &str,
    resource_name: &str,
) -> String {
    format!("/meta/mqtt/blacklist/{cluster_name}/{black_list_type}/{resource_name}")
}

#[inline]
pub fn storage_key_mqtt_blacklist_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/blacklist/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_topic_rewrite_rule(
    cluster_name: &str,
    action: &str,
    source_topic: &str,
) -> String {
    format!("/meta/mqtt/topic_rewrite_rule/{cluster_name}/{action}/{source_topic}")
}

#[inline]
pub fn storage_key_mqtt_topic_rewrite_rule_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/topic_rewrite_rule/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_auto_subscribe_rule(cluster_name: &str, topic: &str) -> String {
    format!("/meta/mqtt/auto_subscribe_rule/{cluster_name}/{topic}")
}

#[inline]
pub fn storage_key_mqtt_auto_subscribe_rule_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/auto_subscribe_rule/{cluster_name}/")
}

#[inline]
pub fn storage_key_mqtt_retain_message(cluster_name: &str, topic_name: &str) -> String {
    format!("/meta/mqtt/retain_message/{cluster_name}/{topic_name}")
}

#[inline]
pub fn storage_key_mqtt_retain_message_cluster_prefix(cluster_name: &str) -> String {
    format!("/meta/mqtt/retain_message/{cluster_name}/")
}
