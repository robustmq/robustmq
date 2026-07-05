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

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::dynamic_config::{
    find_broker_config, find_topic_config, ConfigResourceType, DynamicConfigKey, BROKER_CONFIGS,
    TOPIC_CONFIGS,
};
use crate::handler::tenant::get_tenant;
use common_base::utils::serialize::{deserialize, serialize};
use common_config::broker::broker_config;
use grpc_clients::meta::common::call::{get_resource_config, set_resource_config};
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::alter_configs_request::AlterConfigsResource;
use kafka_protocol::messages::alter_configs_response::AlterConfigsResourceResponse;
use kafka_protocol::messages::describe_configs_request::DescribeConfigsResource;
use kafka_protocol::messages::describe_configs_response::{
    DescribeConfigsResourceResult, DescribeConfigsResult,
};
use kafka_protocol::messages::incremental_alter_configs_request::{
    AlterConfigsResource as IncrementalAlterConfigsResource, AlterableConfig,
};
use kafka_protocol::messages::incremental_alter_configs_response::AlterConfigsResourceResponse as IncrementalAlterConfigsResourceResponse;
use kafka_protocol::messages::{
    AlterConfigsRequest, AlterConfigsResponse, DescribeConfigsRequest, DescribeConfigsResponse,
    IncrementalAlterConfigsRequest, IncrementalAlterConfigsResponse,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use protocol::meta::meta_service_common::{GetResourceConfigRequest, SetResourceConfigRequest};
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

// `org.apache.kafka.clients.admin.ConfigEntry.ConfigSource` wire ordinals.
// Not modeled by the `kafka-protocol` crate (no enum, just a raw i8) and
// purely informational — a wrong value here doesn't break reading the
// config, just mislabels where a tool says the value came from.
const CONFIG_SOURCE_DYNAMIC_TOPIC: i8 = 0;
const CONFIG_SOURCE_DYNAMIC_BROKER_LOGGER: i8 = 1;
const CONFIG_SOURCE_DYNAMIC_BROKER: i8 = 2;
const CONFIG_SOURCE_DEFAULT: i8 = 5;
const CONFIG_SOURCE_UNKNOWN: i8 = -1;

/// Build one config entry: prefer the stored (dynamically-set) value, fall
/// back to the static default from `core::dynamic_config` if this resource
/// never had it set, or report it as present-with-no-value (`UNKNOWN`
/// source) if it's neither stored nor a recognized static config — only
/// reachable for `BrokerLogger`, whose logger names are an open set with no
/// static list to fall back to.
fn build_config_entry(
    name: &str,
    spec: Option<&DynamicConfigKey>,
    stored: &HashMap<String, String>,
    dynamic_source: i8,
    include_documentation: bool,
) -> DescribeConfigsResourceResult {
    let (value, source) = match stored.get(name) {
        Some(v) => (Some(v.clone()), dynamic_source),
        None => match spec {
            Some(s) => (Some(s.default.to_string()), CONFIG_SOURCE_DEFAULT),
            None => (None, CONFIG_SOURCE_UNKNOWN),
        },
    };

    let documentation = if include_documentation {
        spec.map(|s| StrBytes::from_static_str(s.description))
    } else {
        None
    };

    DescribeConfigsResourceResult::default()
        .with_name(StrBytes::from(name.to_string()))
        .with_value(value.map(StrBytes::from))
        .with_config_source(source)
        .with_documentation(documentation)
}

/// Read back one `DescribeConfigsResource`. Unlike Alter/IncrementalAlter,
/// there's no per-config error field in the response — an explicitly
/// requested config name that's neither stored nor a recognized static
/// config (Topic/Broker only; BrokerLogger names are always accepted) is
/// silently omitted from the result rather than failing the whole resource.
async fn describe_one_resource(
    sdm: &Arc<StorageDriverManager>,
    resource: &DescribeConfigsResource,
    include_documentation: bool,
) -> DescribeConfigsResult {
    let resource_name = resource.resource_name.to_string();
    let base = DescribeConfigsResult::default()
        .with_resource_type(resource.resource_type)
        .with_resource_name(resource.resource_name.clone());

    let Some(resource_type) = ConfigResourceType::from_wire(resource.resource_type) else {
        return base
            .with_error_code(ResponseError::InvalidRequest.code())
            .with_error_message(Some(StrBytes::from_static_str(
                "Unknown config resource type",
            )));
    };

    if resource_type == ConfigResourceType::Topic
        && sdm
            .broker_cache
            .get_topic_by_name(get_tenant(), &resource_name)
            .is_none()
    {
        return base.with_error_code(ResponseError::UnknownTopicOrPartition.code());
    }

    let (resource_key, _) = resource_key_and_validator(resource_type, &resource_name);
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let stored: HashMap<String, String> = match get_resource_config(
        client_pool,
        &addrs,
        GetResourceConfigRequest {
            resources: resource_key,
        },
    )
    .await
    {
        Ok(reply) if reply.config.is_empty() => HashMap::new(),
        Ok(reply) => match deserialize(&reply.config) {
            Ok(map) => map,
            Err(e) => {
                warn!(
                    "Kafka DescribeConfigs failed to decode stored config for '{}': {}",
                    resource_name, e
                );
                return base.with_error_code(ResponseError::UnknownServerError.code());
            }
        },
        Err(e) => {
            warn!(
                "Kafka DescribeConfigs storage error reading '{}': {}",
                resource_name, e
            );
            return base.with_error_code(ResponseError::UnknownServerError.code());
        }
    };

    let dynamic_source = match resource_type {
        ConfigResourceType::Topic => CONFIG_SOURCE_DYNAMIC_TOPIC,
        ConfigResourceType::Broker => CONFIG_SOURCE_DYNAMIC_BROKER,
        ConfigResourceType::BrokerLogger => CONFIG_SOURCE_DYNAMIC_BROKER_LOGGER,
    };
    let static_list: &[DynamicConfigKey] = match resource_type {
        ConfigResourceType::Topic => TOPIC_CONFIGS,
        ConfigResourceType::Broker => BROKER_CONFIGS,
        ConfigResourceType::BrokerLogger => &[],
    };

    let configs = match &resource.configuration_keys {
        Some(names) => names
            .iter()
            .filter_map(|name| {
                let name = name.as_str();
                let spec = static_list.iter().find(|s| s.name == name);
                if resource_type != ConfigResourceType::BrokerLogger
                    && spec.is_none()
                    && !stored.contains_key(name)
                {
                    return None;
                }
                Some(build_config_entry(
                    name,
                    spec,
                    &stored,
                    dynamic_source,
                    include_documentation,
                ))
            })
            .collect(),
        // "List all": for Topic/Broker that means every statically-known
        // config; BrokerLogger has no static list, so it's whatever this
        // resource actually has a level set for.
        None => match resource_type {
            ConfigResourceType::BrokerLogger => stored
                .keys()
                .map(|name| {
                    build_config_entry(name, None, &stored, dynamic_source, include_documentation)
                })
                .collect(),
            _ => static_list
                .iter()
                .map(|spec| {
                    build_config_entry(
                        spec.name,
                        Some(spec),
                        &stored,
                        dynamic_source,
                        include_documentation,
                    )
                })
                .collect(),
        },
    };

    base.with_error_code(0).with_configs(configs)
}

pub async fn process_describe_configs(
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeConfigsRequest,
) -> Option<KafkaPacket> {
    let mut results = Vec::with_capacity(req.resources.len());
    for resource in &req.resources {
        results.push(describe_one_resource(sdm, resource, req.include_documentation).await);
    }

    Some(KafkaPacket::DescribeConfigsResponse(
        DescribeConfigsResponse::default().with_results(results),
    ))
}

/// Resource-key prefix under which this resource's config blob is stored,
/// plus a name validator for `AlterConfigsResource.configs`. `BrokerLogger`
/// has no fixed name list — logger names are an open set, so any name is
/// accepted.
fn resource_key_and_validator(
    resource_type: ConfigResourceType,
    resource_name: &str,
) -> (Vec<String>, fn(&str) -> bool) {
    match resource_type {
        ConfigResourceType::Topic => (
            vec![
                "kafka".to_string(),
                "topic".to_string(),
                resource_name.to_string(),
            ],
            |name| find_topic_config(name).is_some(),
        ),
        ConfigResourceType::Broker => (
            vec![
                "kafka".to_string(),
                "broker".to_string(),
                resource_name.to_string(),
            ],
            |name| find_broker_config(name).is_some(),
        ),
        ConfigResourceType::BrokerLogger => (
            vec![
                "kafka".to_string(),
                "broker_logger".to_string(),
                resource_name.to_string(),
            ],
            |_name| true,
        ),
    }
}

/// Apply one `AlterConfigsResource`: validate resource type, topic
/// existence (Topic only), and every config name, then persist the full
/// requested config set as a single JSON-ish blob (bincode) at this
/// resource's key. This is a wholesale replace — matching `AlterConfigs`
/// semantics — not a merge with whatever was stored before.
///
/// Storage only: none of these values are wired to anything that changes
/// broker behavior yet (see `core::dynamic_config` for which config names
/// have a real RobustMQ field to eventually apply to).
async fn alter_one_resource(
    sdm: &Arc<StorageDriverManager>,
    resource: &AlterConfigsResource,
    validate_only: bool,
) -> AlterConfigsResourceResponse {
    let resource_name = resource.resource_name.to_string();
    let base = AlterConfigsResourceResponse::default()
        .with_resource_type(resource.resource_type)
        .with_resource_name(resource.resource_name.clone());

    let Some(resource_type) = ConfigResourceType::from_wire(resource.resource_type) else {
        return base
            .with_error_code(ResponseError::InvalidRequest.code())
            .with_error_message(Some(StrBytes::from_static_str(
                "Unknown config resource type",
            )));
    };

    if resource_type == ConfigResourceType::Topic
        && sdm
            .broker_cache
            .get_topic_by_name(get_tenant(), &resource_name)
            .is_none()
    {
        return base.with_error_code(ResponseError::UnknownTopicOrPartition.code());
    }

    let (resource_key, name_is_known) = resource_key_and_validator(resource_type, &resource_name);

    if let Some(unknown) = resource.configs.iter().find(|c| !name_is_known(&c.name)) {
        warn!(
            "Kafka AlterConfigs rejected unknown config '{}' for resource '{}'",
            unknown.name, resource_name
        );
        return base
            .with_error_code(ResponseError::InvalidConfig.code())
            .with_error_message(Some(StrBytes::from_static_str(
                "Unknown configuration name",
            )));
    }

    if validate_only {
        return base.with_error_code(0);
    }

    // A config entry with no value means "reset to default" (old
    // AlterConfigs semantics) — simply omitted from the stored set.
    let config_map: HashMap<String, String> = resource
        .configs
        .iter()
        .filter_map(|c| {
            c.value
                .as_ref()
                .map(|v| (c.name.to_string(), v.to_string()))
        })
        .collect();

    let bytes = match serialize(&config_map) {
        Ok(b) => b,
        Err(e) => {
            warn!(
                "Kafka AlterConfigs failed to serialize config for '{}': {}",
                resource_name, e
            );
            return base.with_error_code(ResponseError::UnknownServerError.code());
        }
    };

    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();
    let request = SetResourceConfigRequest {
        resources: resource_key,
        config: bytes,
    };
    if let Err(e) = set_resource_config(client_pool, &addrs, request).await {
        warn!(
            "Kafka AlterConfigs storage error for '{}': {}",
            resource_name, e
        );
        return base.with_error_code(ResponseError::UnknownServerError.code());
    }

    base.with_error_code(0)
}

pub async fn process_alter_configs(
    sdm: &Arc<StorageDriverManager>,
    req: &AlterConfigsRequest,
) -> Option<KafkaPacket> {
    let mut responses = Vec::with_capacity(req.resources.len());
    for resource in &req.resources {
        responses.push(alter_one_resource(sdm, resource, req.validate_only).await);
    }

    Some(KafkaPacket::AlterConfigsResponse(
        AlterConfigsResponse::default().with_responses(responses),
    ))
}

/// `AlterConfigOp.OpType` wire values (KIP-248). `Append`/`Subtract` only
/// make sense for list-valued configs; RobustMQ doesn't track which stored
/// configs are list-typed, so those two are rejected rather than silently
/// doing something wrong to an opaque string value.
const CONFIG_OP_SET: i8 = 0;
const CONFIG_OP_DELETE: i8 = 1;

/// Apply a validated list of SET/DELETE operations onto an existing config
/// map in place. Pulled out of `incremental_alter_one_resource` so the
/// actual mutation logic is testable without a `StorageDriverManager`.
fn apply_incremental_ops(config_map: &mut HashMap<String, String>, configs: &[AlterableConfig]) {
    for c in configs {
        match c.config_operation {
            CONFIG_OP_SET => {
                if let Some(value) = &c.value {
                    config_map.insert(c.name.to_string(), value.to_string());
                }
            }
            CONFIG_OP_DELETE => {
                config_map.remove(c.name.as_str());
            }
            _ => unreachable!("validated above: only SET/DELETE reach this point"),
        }
    }
}

/// Read-modify-write version of `alter_one_resource`: apply only the
/// requested SET/DELETE operations on top of whatever config is already
/// stored for this resource, instead of replacing the whole set.
async fn incremental_alter_one_resource(
    sdm: &Arc<StorageDriverManager>,
    resource: &IncrementalAlterConfigsResource,
    validate_only: bool,
) -> IncrementalAlterConfigsResourceResponse {
    let resource_name = resource.resource_name.to_string();
    let base = IncrementalAlterConfigsResourceResponse::default()
        .with_resource_type(resource.resource_type)
        .with_resource_name(resource.resource_name.clone());

    let Some(resource_type) = ConfigResourceType::from_wire(resource.resource_type) else {
        return base
            .with_error_code(ResponseError::InvalidRequest.code())
            .with_error_message(Some(StrBytes::from_static_str(
                "Unknown config resource type",
            )));
    };

    if resource_type == ConfigResourceType::Topic
        && sdm
            .broker_cache
            .get_topic_by_name(get_tenant(), &resource_name)
            .is_none()
    {
        return base.with_error_code(ResponseError::UnknownTopicOrPartition.code());
    }

    let (resource_key, name_is_known) = resource_key_and_validator(resource_type, &resource_name);

    if let Some(unknown) = resource.configs.iter().find(|c| !name_is_known(&c.name)) {
        warn!(
            "Kafka IncrementalAlterConfigs rejected unknown config '{}' for resource '{}'",
            unknown.name, resource_name
        );
        return base
            .with_error_code(ResponseError::InvalidConfig.code())
            .with_error_message(Some(StrBytes::from_static_str(
                "Unknown configuration name",
            )));
    }

    if let Some(unsupported) = resource
        .configs
        .iter()
        .find(|c| c.config_operation != CONFIG_OP_SET && c.config_operation != CONFIG_OP_DELETE)
    {
        warn!(
            "Kafka IncrementalAlterConfigs rejected operation {} for config '{}' on resource '{}': only SET/DELETE are supported",
            unsupported.config_operation, unsupported.name, resource_name
        );
        return base
            .with_error_code(ResponseError::InvalidRequest.code())
            .with_error_message(Some(StrBytes::from_static_str(
                "Only SET and DELETE operations are supported",
            )));
    }

    if validate_only {
        return base.with_error_code(0);
    }

    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let mut config_map: HashMap<String, String> = match get_resource_config(
        client_pool,
        &addrs,
        GetResourceConfigRequest {
            resources: resource_key.clone(),
        },
    )
    .await
    {
        Ok(reply) if reply.config.is_empty() => HashMap::new(),
        Ok(reply) => match deserialize(&reply.config) {
            Ok(map) => map,
            Err(e) => {
                warn!(
                    "Kafka IncrementalAlterConfigs failed to decode stored config for '{}': {}",
                    resource_name, e
                );
                return base.with_error_code(ResponseError::UnknownServerError.code());
            }
        },
        Err(e) => {
            warn!(
                "Kafka IncrementalAlterConfigs storage error reading '{}': {}",
                resource_name, e
            );
            return base.with_error_code(ResponseError::UnknownServerError.code());
        }
    };

    apply_incremental_ops(&mut config_map, &resource.configs);

    let bytes = match serialize(&config_map) {
        Ok(b) => b,
        Err(e) => {
            warn!(
                "Kafka IncrementalAlterConfigs failed to serialize config for '{}': {}",
                resource_name, e
            );
            return base.with_error_code(ResponseError::UnknownServerError.code());
        }
    };

    let request = SetResourceConfigRequest {
        resources: resource_key,
        config: bytes,
    };
    if let Err(e) = set_resource_config(client_pool, &addrs, request).await {
        warn!(
            "Kafka IncrementalAlterConfigs storage error writing '{}': {}",
            resource_name, e
        );
        return base.with_error_code(ResponseError::UnknownServerError.code());
    }

    base.with_error_code(0)
}

pub async fn process_incremental_alter_configs(
    sdm: &Arc<StorageDriverManager>,
    req: &IncrementalAlterConfigsRequest,
) -> Option<KafkaPacket> {
    let mut responses = Vec::with_capacity(req.resources.len());
    for resource in &req.resources {
        responses.push(incremental_alter_one_resource(sdm, resource, req.validate_only).await);
    }

    Some(KafkaPacket::IncrementalAlterConfigsResponse(
        IncrementalAlterConfigsResponse::default().with_responses(responses),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_key_and_validator_scopes_topic_configs_under_topic_name() {
        let (key, is_known) = resource_key_and_validator(ConfigResourceType::Topic, "orders");
        assert_eq!(key, vec!["kafka", "topic", "orders"]);
        assert!(is_known("retention.ms"));
        assert!(is_known("cleanup.policy"));
        assert!(!is_known("not.a.real.config"));
    }

    #[test]
    fn resource_key_and_validator_broker_empty_name_is_cluster_default() {
        let (key, is_known) = resource_key_and_validator(ConfigResourceType::Broker, "");
        assert_eq!(key, vec!["kafka", "broker", ""]);
        assert!(is_known("log.retention.ms"));
        assert!(!is_known("retention.ms")); // topic-only config, not a broker config
    }

    #[test]
    fn resource_key_and_validator_broker_logger_accepts_any_name() {
        let (key, is_known) = resource_key_and_validator(ConfigResourceType::BrokerLogger, "1");
        assert_eq!(key, vec!["kafka", "broker_logger", "1"]);
        assert!(is_known("kafka.controller"));
        assert!(is_known("anything.at.all"));
    }

    #[test]
    fn build_config_entry_prefers_stored_value_over_default() {
        let spec = find_topic_config("retention.ms").unwrap();
        let mut stored = HashMap::new();
        stored.insert("retention.ms".to_string(), "3600000".to_string());

        let entry = build_config_entry(
            "retention.ms",
            Some(spec),
            &stored,
            CONFIG_SOURCE_DYNAMIC_TOPIC,
            false,
        );
        assert_eq!(entry.value.as_deref(), Some("3600000"));
        assert_eq!(entry.config_source, CONFIG_SOURCE_DYNAMIC_TOPIC);
    }

    #[test]
    fn build_config_entry_falls_back_to_static_default_when_unstored() {
        let spec = find_topic_config("retention.ms").unwrap();
        let stored = HashMap::new();

        let entry = build_config_entry(
            "retention.ms",
            Some(spec),
            &stored,
            CONFIG_SOURCE_DYNAMIC_TOPIC,
            false,
        );
        assert_eq!(entry.value.as_deref(), Some(spec.default));
        assert_eq!(entry.config_source, CONFIG_SOURCE_DEFAULT);
    }

    #[test]
    fn build_config_entry_reports_unknown_source_when_neither_stored_nor_static() {
        // The BrokerLogger case: a logger name that hasn't had a level set.
        let stored = HashMap::new();
        let entry = build_config_entry(
            "kafka.controller",
            None,
            &stored,
            CONFIG_SOURCE_DYNAMIC_BROKER_LOGGER,
            false,
        );
        assert_eq!(entry.value, None);
        assert_eq!(entry.config_source, CONFIG_SOURCE_UNKNOWN);
    }

    #[test]
    fn build_config_entry_includes_documentation_only_when_requested() {
        let spec = find_topic_config("retention.ms").unwrap();
        let stored = HashMap::new();

        let without_docs = build_config_entry(
            "retention.ms",
            Some(spec),
            &stored,
            CONFIG_SOURCE_DYNAMIC_TOPIC,
            false,
        );
        assert_eq!(without_docs.documentation, None);

        let with_docs = build_config_entry(
            "retention.ms",
            Some(spec),
            &stored,
            CONFIG_SOURCE_DYNAMIC_TOPIC,
            true,
        );
        assert_eq!(with_docs.documentation.as_deref(), Some(spec.description));
    }

    fn alterable_config(name: &str, op: i8, value: Option<&str>) -> AlterableConfig {
        AlterableConfig::default()
            .with_name(StrBytes::from(name.to_string()))
            .with_config_operation(op)
            .with_value(value.map(|v| StrBytes::from(v.to_string())))
    }

    #[test]
    fn apply_incremental_ops_set_inserts_and_overwrites() {
        let mut map = HashMap::new();
        map.insert("min.insync.replicas".to_string(), "1".to_string());

        let ops = vec![
            alterable_config("min.insync.replicas", CONFIG_OP_SET, Some("2")),
            alterable_config("retention.ms", CONFIG_OP_SET, Some("3600000")),
        ];
        apply_incremental_ops(&mut map, &ops);

        assert_eq!(
            map.get("min.insync.replicas").map(String::as_str),
            Some("2")
        );
        assert_eq!(map.get("retention.ms").map(String::as_str), Some("3600000"));
    }

    #[test]
    fn apply_incremental_ops_delete_removes_key() {
        let mut map = HashMap::new();
        map.insert("retention.ms".to_string(), "3600000".to_string());

        apply_incremental_ops(
            &mut map,
            &[alterable_config("retention.ms", CONFIG_OP_DELETE, None)],
        );

        assert!(!map.contains_key("retention.ms"));
    }

    #[test]
    fn apply_incremental_ops_set_with_no_value_is_a_no_op() {
        let mut map = HashMap::new();

        apply_incremental_ops(
            &mut map,
            &[alterable_config("retention.ms", CONFIG_OP_SET, None)],
        );

        assert!(map.is_empty());
    }

    #[test]
    fn apply_incremental_ops_delete_on_missing_key_is_a_no_op() {
        let mut map = HashMap::new();

        apply_incremental_ops(
            &mut map,
            &[alterable_config("retention.ms", CONFIG_OP_DELETE, None)],
        );

        assert!(map.is_empty());
    }
}
