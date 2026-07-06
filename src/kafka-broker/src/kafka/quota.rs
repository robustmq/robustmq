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

use crate::handler::tenant::get_tenant;
use common_config::broker::broker_config;
use grpc_clients::meta::kafka::call::{delete_kafka_quota, list_kafka_quota, set_kafka_quota};
use grpc_clients::pool::ClientPool;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::alter_client_quotas_request::EntryData as AlterEntry;
use kafka_protocol::messages::alter_client_quotas_response::{
    EntityData as AlterRespEntity, EntryData as AlterRespEntry,
};
use kafka_protocol::messages::describe_client_quotas_response::{
    EntityData as DescribeEntity, EntryData as DescribeEntry, ValueData,
};
use kafka_protocol::messages::{
    AlterClientQuotasRequest, AlterClientQuotasResponse, DescribeClientQuotasRequest,
    DescribeClientQuotasResponse,
};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::kafka::quota::{
    KafkaClientQuota, QUOTA_DEFAULT_NAME, QUOTA_ENTITY_CLIENT_ID, QUOTA_KEY_CONSUMER_BYTE_RATE,
    QUOTA_KEY_PRODUCER_BYTE_RATE,
};
use protocol::kafka::packet::KafkaPacket;
use protocol::meta::meta_service_kafka::{
    DeleteKafkaQuotaRequest, ListKafkaQuotaRequest, SetKafkaQuotaRequest,
};
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

// DescribeClientQuotas match_type values.
const MATCH_TYPE_EXACT: i8 = 0;
const MATCH_TYPE_DEFAULT: i8 = 1;
const MATCH_TYPE_ANY: i8 = 2;

const SUPPORTED_QUOTA_KEYS: [&str; 2] =
    [QUOTA_KEY_PRODUCER_BYTE_RATE, QUOTA_KEY_CONSUMER_BYTE_RATE];

// Validate one Alter entry down to (entity_name, ops); only single-dimension
// client-id entities and the supported quota keys are representable.
fn validate_entry(entry: &AlterEntry) -> Result<Option<String>, (i16, String)> {
    let invalid = |msg: String| (ResponseError::InvalidRequest.code(), msg);
    if entry.entity.len() != 1 {
        return Err(invalid(
            "only single-dimension quota entities are supported".to_string(),
        ));
    }
    let entity = &entry.entity[0];
    if entity.entity_type.as_str() != QUOTA_ENTITY_CLIENT_ID {
        return Err(invalid(format!(
            "unsupported quota entity type: {}",
            entity.entity_type.as_str()
        )));
    }
    if let Some(name) = &entity.entity_name {
        if name.is_empty() {
            return Err(invalid("entity name must not be empty".to_string()));
        }
        if name.as_str() == QUOTA_DEFAULT_NAME {
            return Err(invalid(format!(
                "entity name {} is reserved",
                QUOTA_DEFAULT_NAME
            )));
        }
    }
    for op in &entry.ops {
        if !SUPPORTED_QUOTA_KEYS.contains(&op.key.as_str()) {
            return Err(invalid(format!(
                "unsupported quota key: {}",
                op.key.as_str()
            )));
        }
        if !op.remove && (op.value <= 0.0 || !op.value.is_finite()) {
            return Err(invalid(format!(
                "invalid quota value for {}",
                op.key.as_str()
            )));
        }
    }
    Ok(entity.entity_name.as_ref().map(|s| s.to_string()))
}

// Merge the entry's ops into the currently stored quota (upsert keys, remove
// keys) and return the resulting record; an empty record means "delete".
fn merge_ops(
    existing: Option<KafkaClientQuota>,
    entity_name: Option<String>,
    entry: &AlterEntry,
) -> KafkaClientQuota {
    let mut quota = existing.unwrap_or_else(|| KafkaClientQuota {
        tenant: get_tenant().to_string(),
        entity_type: QUOTA_ENTITY_CLIENT_ID.to_string(),
        entity_name,
        quotas: Default::default(),
    });
    for op in &entry.ops {
        if op.remove {
            quota.quotas.remove(op.key.as_str());
        } else {
            quota.quotas.insert(op.key.to_string(), op.value);
        }
    }
    quota
}

async fn persist_quota(
    client_pool: &Arc<ClientPool>,
    addrs: &[String],
    quota: &KafkaClientQuota,
) -> Result<(), String> {
    if quota.quotas.is_empty() {
        let request = DeleteKafkaQuotaRequest {
            tenant: quota.tenant.clone(),
            entity_type: quota.entity_type.clone(),
            entity_name: quota.name_key().to_string(),
        };
        delete_kafka_quota(client_pool, addrs, request)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        let request = SetKafkaQuotaRequest {
            quota: quota.encode().map_err(|e| e.to_string())?,
        };
        set_kafka_quota(client_pool, addrs, request)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn list_quotas(sdm: &Arc<StorageDriverManager>) -> Result<Vec<KafkaClientQuota>, String> {
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();
    let reply = list_kafka_quota(
        client_pool,
        &addrs,
        ListKafkaQuotaRequest {
            tenant: get_tenant().to_string(),
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut quotas = Vec::with_capacity(reply.quotas.len());
    for raw in reply.quotas {
        quotas.push(KafkaClientQuota::decode(&raw).map_err(|e| e.to_string())?);
    }
    Ok(quotas)
}

pub async fn process_alter_client_quotas(
    sdm: &Arc<StorageDriverManager>,
    req: &AlterClientQuotasRequest,
) -> Option<KafkaPacket> {
    let stored = match list_quotas(sdm).await {
        Ok(quotas) => quotas,
        Err(e) => {
            warn!("Kafka AlterClientQuotas failed to list quotas: {}", e);
            let entries = req
                .entries
                .iter()
                .map(|entry| {
                    entry_response(
                        entry,
                        ResponseError::UnknownServerError.code(),
                        Some(e.clone()),
                    )
                })
                .collect();
            return Some(KafkaPacket::AlterClientQuotasResponse(
                AlterClientQuotasResponse::default().with_entries(entries),
            ));
        }
    };

    let client_pool = sdm.engine_storage_handler.client_pool.clone();
    let addrs = broker_config().get_meta_service_addr();

    // Track the effective state across entries: two entries touching the same
    // entity within one request must see each other's changes.
    let mut current: HashMap<Option<String>, KafkaClientQuota> = stored
        .into_iter()
        .filter(|q| q.entity_type == QUOTA_ENTITY_CLIENT_ID)
        .map(|q| (q.entity_name.clone(), q))
        .collect();

    let mut entries = Vec::with_capacity(req.entries.len());
    for entry in &req.entries {
        let result: Result<(), (i16, String)> = match validate_entry(entry) {
            Ok(entity_name) => {
                if req.validate_only {
                    Ok(())
                } else {
                    let merged = merge_ops(
                        current.get(&entity_name).cloned(),
                        entity_name.clone(),
                        entry,
                    );
                    match persist_quota(&client_pool, &addrs, &merged).await {
                        Ok(()) => {
                            if merged.quotas.is_empty() {
                                current.remove(&entity_name);
                            } else {
                                current.insert(entity_name, merged);
                            }
                            Ok(())
                        }
                        Err(e) => {
                            warn!("Kafka AlterClientQuotas failed to persist: {}", e);
                            Err((ResponseError::UnknownServerError.code(), e))
                        }
                    }
                }
            }
            Err(e) => Err(e),
        };

        let (code, message) = match result {
            Ok(()) => (0, None),
            Err((code, message)) => (code, Some(message)),
        };
        entries.push(entry_response(entry, code, message));
    }

    Some(KafkaPacket::AlterClientQuotasResponse(
        AlterClientQuotasResponse::default().with_entries(entries),
    ))
}

fn entry_response(entry: &AlterEntry, code: i16, message: Option<String>) -> AlterRespEntry {
    let entity = entry
        .entity
        .iter()
        .map(|e| {
            AlterRespEntity::default()
                .with_entity_type(e.entity_type.clone())
                .with_entity_name(e.entity_name.clone())
        })
        .collect();
    AlterRespEntry::default()
        .with_error_code(code)
        .with_error_message(message.map(StrBytes::from))
        .with_entity(entity)
}

// DescribeClientQuotas filter semantics: every component must match the record
// (exact name / the type default / any); with `strict`, the record must not have
// dimensions beyond the filtered ones (ours are all single-dimension, so this
// only rejects filters on other entity types).
fn quota_matches(quota: &KafkaClientQuota, req: &DescribeClientQuotasRequest) -> bool {
    // strict: every dimension the record has must be named in the filter; our
    // records have exactly one dimension (the entity type).
    if req.strict
        && !req
            .components
            .iter()
            .any(|c| c.entity_type.as_str() == quota.entity_type)
    {
        return false;
    }
    for component in &req.components {
        if component.entity_type.as_str() != quota.entity_type {
            return false;
        }
        let matched = match component.match_type {
            MATCH_TYPE_EXACT => {
                let name = component._match.as_ref().map(|s| s.to_string());
                quota.entity_name == name && name.is_some()
            }
            MATCH_TYPE_DEFAULT => quota.entity_name.is_none(),
            MATCH_TYPE_ANY => true,
            _ => false,
        };
        if !matched {
            return false;
        }
    }
    true
}

pub async fn process_describe_client_quotas(
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeClientQuotasRequest,
) -> Option<KafkaPacket> {
    let stored = match list_quotas(sdm).await {
        Ok(quotas) => quotas,
        Err(e) => {
            warn!("Kafka DescribeClientQuotas failed to list quotas: {}", e);
            return Some(KafkaPacket::DescribeClientQuotasResponse(
                DescribeClientQuotasResponse::default()
                    .with_error_code(ResponseError::UnknownServerError.code())
                    .with_error_message(Some(StrBytes::from(e))),
            ));
        }
    };

    let mut entries: Vec<DescribeEntry> = stored
        .iter()
        .filter(|q| quota_matches(q, req))
        .map(|q| {
            let mut values: Vec<ValueData> = q
                .quotas
                .iter()
                .map(|(key, value)| {
                    ValueData::default()
                        .with_key(StrBytes::from(key.clone()))
                        .with_value(*value)
                })
                .collect();
            values.sort_by(|a, b| a.key.cmp(&b.key));

            DescribeEntry::default()
                .with_entity(vec![DescribeEntity::default()
                    .with_entity_type(StrBytes::from(q.entity_type.clone()))
                    .with_entity_name(q.entity_name.clone().map(StrBytes::from))])
                .with_values(values)
        })
        .collect();
    entries.sort_by(|a, b| {
        let name_of = |e: &DescribeEntry| {
            e.entity
                .first()
                .and_then(|en| en.entity_name.as_ref().map(|n| n.to_string()))
                .unwrap_or_default()
        };
        name_of(a).cmp(&name_of(b))
    });

    Some(KafkaPacket::DescribeClientQuotasResponse(
        DescribeClientQuotasResponse::default()
            .with_error_code(0)
            .with_entries(Some(entries)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kafka_protocol::messages::alter_client_quotas_request::{
        EntityData as AlterReqEntity, OpData,
    };
    use kafka_protocol::messages::describe_client_quotas_request::ComponentData;

    fn entry(entity_type: &str, name: Option<&str>, ops: Vec<(&str, f64, bool)>) -> AlterEntry {
        AlterEntry::default()
            .with_entity(vec![AlterReqEntity::default()
                .with_entity_type(StrBytes::from(entity_type.to_string()))
                .with_entity_name(name.map(|n| StrBytes::from(n.to_string())))])
            .with_ops(
                ops.into_iter()
                    .map(|(key, value, remove)| {
                        OpData::default()
                            .with_key(StrBytes::from(key.to_string()))
                            .with_value(value)
                            .with_remove(remove)
                    })
                    .collect(),
            )
    }

    #[test]
    fn validate_entry_accepts_client_id_and_rejects_others() {
        let ok = entry(
            "client-id",
            Some("app"),
            vec![("producer_byte_rate", 1024.0, false)],
        );
        assert_eq!(validate_entry(&ok).unwrap(), Some("app".to_string()));

        let default = entry(
            "client-id",
            None,
            vec![("consumer_byte_rate", 1024.0, false)],
        );
        assert_eq!(validate_entry(&default).unwrap(), None);

        let bad_type = entry("user", Some("u"), vec![("producer_byte_rate", 1.0, false)]);
        assert!(validate_entry(&bad_type).is_err());

        let bad_key = entry(
            "client-id",
            Some("a"),
            vec![("request_percentage", 1.0, false)],
        );
        assert!(validate_entry(&bad_key).is_err());

        let bad_value = entry(
            "client-id",
            Some("a"),
            vec![("producer_byte_rate", 0.0, false)],
        );
        assert!(validate_entry(&bad_value).is_err());

        let empty_name = entry(
            "client-id",
            Some(""),
            vec![("producer_byte_rate", 1.0, false)],
        );
        assert!(validate_entry(&empty_name).is_err());

        let reserved = entry(
            "client-id",
            Some("__default__"),
            vec![("producer_byte_rate", 1.0, false)],
        );
        assert!(validate_entry(&reserved).is_err());
    }

    #[test]
    fn strict_describe_requires_dimensions_in_filter() {
        let named = quota(Some("app"));
        let strict_with_component = describe_req(MATCH_TYPE_ANY, None).with_strict(true);
        assert!(quota_matches(&named, &strict_with_component));

        let strict_empty = DescribeClientQuotasRequest::default().with_strict(true);
        assert!(!quota_matches(&named, &strict_empty));
    }

    #[test]
    fn merge_ops_upserts_and_removes_keys() {
        let e = entry(
            "client-id",
            Some("app"),
            vec![
                ("producer_byte_rate", 2048.0, false),
                ("consumer_byte_rate", 0.0, true),
            ],
        );
        let existing = KafkaClientQuota {
            tenant: "default".to_string(),
            entity_type: "client-id".to_string(),
            entity_name: Some("app".to_string()),
            quotas: [
                ("producer_byte_rate".to_string(), 1024.0),
                ("consumer_byte_rate".to_string(), 512.0),
            ]
            .into(),
        };

        let merged = merge_ops(Some(existing), Some("app".to_string()), &e);
        assert_eq!(merged.quotas["producer_byte_rate"], 2048.0);
        assert!(!merged.quotas.contains_key("consumer_byte_rate"));

        // Removing the last key empties the record (caller then deletes it).
        let remove_all = entry(
            "client-id",
            Some("app"),
            vec![("producer_byte_rate", 0.0, true)],
        );
        let merged = merge_ops(Some(merged), Some("app".to_string()), &remove_all);
        assert!(merged.quotas.is_empty());
    }

    fn quota(name: Option<&str>) -> KafkaClientQuota {
        KafkaClientQuota {
            tenant: "default".to_string(),
            entity_type: "client-id".to_string(),
            entity_name: name.map(|n| n.to_string()),
            quotas: [("producer_byte_rate".to_string(), 1.0)].into(),
        }
    }

    fn describe_req(match_type: i8, name: Option<&str>) -> DescribeClientQuotasRequest {
        DescribeClientQuotasRequest::default().with_components(vec![ComponentData::default()
            .with_entity_type(StrBytes::from_static_str("client-id"))
            .with_match_type(match_type)
            .with_match(name.map(|n| StrBytes::from(n.to_string())))])
    }

    #[test]
    fn describe_filter_semantics() {
        let named = quota(Some("app"));
        let default = quota(None);

        assert!(quota_matches(
            &named,
            &describe_req(MATCH_TYPE_EXACT, Some("app"))
        ));
        assert!(!quota_matches(
            &named,
            &describe_req(MATCH_TYPE_EXACT, Some("other"))
        ));
        assert!(!quota_matches(
            &default,
            &describe_req(MATCH_TYPE_EXACT, Some("app"))
        ));

        assert!(quota_matches(
            &default,
            &describe_req(MATCH_TYPE_DEFAULT, None)
        ));
        assert!(!quota_matches(
            &named,
            &describe_req(MATCH_TYPE_DEFAULT, None)
        ));

        assert!(quota_matches(&named, &describe_req(MATCH_TYPE_ANY, None)));
        assert!(quota_matches(&default, &describe_req(MATCH_TYPE_ANY, None)));

        // No components = match everything.
        let empty = DescribeClientQuotasRequest::default();
        assert!(quota_matches(&named, &empty));
    }
}
