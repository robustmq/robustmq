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

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::core::acl::{binding_matches, security_acl_to_binding, to_security_acl};
use common_security::storage::acl::AclStorage;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::create_acls_response::AclCreationResult;
use kafka_protocol::messages::delete_acls_response::{
    DeleteAclsFilterResult, DeleteAclsMatchingAcl,
};
use kafka_protocol::messages::describe_acls_response::{AclDescription, DescribeAclsResource};
use kafka_protocol::messages::{
    CreateAclsRequest, CreateAclsResponse, DeleteAclsRequest, DeleteAclsResponse,
    DescribeAclsRequest, DescribeAclsResponse,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

pub async fn process_describe_acls(
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeAclsRequest,
) -> Option<KafkaPacket> {
    let acl_storage = AclStorage::new(sdm.engine_storage_handler.client_pool.clone());
    let acls = match acl_storage.list_acl().await {
        Ok(acls) => acls,
        Err(e) => {
            warn!("Kafka DescribeAcls failed to list ACLs: {}", e);
            return Some(KafkaPacket::DescribeAclsResponse(
                DescribeAclsResponse::default()
                    .with_error_code(ResponseError::UnknownServerError.code())
                    .with_error_message(Some(StrBytes::from(e.to_string()))),
            ));
        }
    };

    // Group matched bindings by resource (BTreeMap keeps the response deterministic).
    let mut grouped: BTreeMap<(i8, String, i8), Vec<AclDescription>> = BTreeMap::new();
    for acl in &acls {
        let Some(b) = security_acl_to_binding(acl) else {
            continue;
        };
        if !binding_matches(
            &b,
            req.resource_type_filter,
            req.resource_name_filter.as_ref(),
            req.pattern_type_filter,
            req.principal_filter.as_ref(),
            req.host_filter.as_ref(),
            req.operation,
            req.permission_type,
        ) {
            continue;
        }
        grouped
            .entry((b.resource_type, b.resource_name.clone(), b.pattern_type))
            .or_default()
            .push(
                AclDescription::default()
                    .with_principal(StrBytes::from(b.principal))
                    .with_host(StrBytes::from(b.host))
                    .with_operation(b.operation)
                    .with_permission_type(b.permission_type),
            );
    }

    let resources = grouped
        .into_iter()
        .map(|((rt, rname, pt), acls)| {
            DescribeAclsResource::default()
                .with_resource_type(rt)
                .with_resource_name(StrBytes::from(rname))
                .with_pattern_type(pt)
                .with_acls(acls)
        })
        .collect();

    Some(KafkaPacket::DescribeAclsResponse(
        DescribeAclsResponse::default()
            .with_error_code(0)
            .with_resources(resources),
    ))
}

pub async fn process_create_acls(
    sdm: &Arc<StorageDriverManager>,
    req: &CreateAclsRequest,
) -> Option<KafkaPacket> {
    let acl_storage = AclStorage::new(sdm.engine_storage_handler.client_pool.clone());
    let mut results = Vec::with_capacity(req.creations.len());

    for creation in &req.creations {
        let result = match to_security_acl(creation) {
            Ok(acl) => match acl_storage.save_acl(acl).await {
                Ok(()) => AclCreationResult::default().with_error_code(0),
                Err(e) => {
                    warn!("Kafka CreateAcls failed to persist ACL: {}", e);
                    AclCreationResult::default()
                        .with_error_code(ResponseError::UnknownServerError.code())
                        .with_error_message(Some(StrBytes::from(e.to_string())))
                }
            },
            Err(err) => AclCreationResult::default()
                .with_error_code(err.code())
                .with_error_message(Some(StrBytes::from(format!("{err:?}")))),
        };
        results.push(result);
    }

    Some(KafkaPacket::CreateAclsResponse(
        CreateAclsResponse::default().with_results(results),
    ))
}

pub async fn process_delete_acls(
    sdm: &Arc<StorageDriverManager>,
    req: &DeleteAclsRequest,
) -> Option<KafkaPacket> {
    let acl_storage = AclStorage::new(sdm.engine_storage_handler.client_pool.clone());
    let acls = match acl_storage.list_acl().await {
        Ok(acls) => acls,
        Err(e) => {
            warn!("Kafka DeleteAcls failed to list ACLs: {}", e);
            let filter_results = req
                .filters
                .iter()
                .map(|_| {
                    DeleteAclsFilterResult::default()
                        .with_error_code(ResponseError::UnknownServerError.code())
                        .with_error_message(Some(StrBytes::from(e.to_string())))
                })
                .collect();
            return Some(KafkaPacket::DeleteAclsResponse(
                DeleteAclsResponse::default().with_filter_results(filter_results),
            ));
        }
    };

    // A single ACL matched by several filters is deleted once; later filters still
    // report it as matched (with success), which mirrors Kafka's per-filter results.
    let mut deleted: HashSet<String> = HashSet::new();
    let mut filter_results = Vec::with_capacity(req.filters.len());

    for filter in &req.filters {
        let mut matching = Vec::new();
        for acl in &acls {
            let Some(b) = security_acl_to_binding(acl) else {
                continue;
            };
            if !binding_matches(
                &b,
                filter.resource_type_filter,
                filter.resource_name_filter.as_ref(),
                filter.pattern_type_filter,
                filter.principal_filter.as_ref(),
                filter.host_filter.as_ref(),
                filter.operation,
                filter.permission_type,
            ) {
                continue;
            }

            let (mut error_code, mut error_message) = (0, None);
            if !deleted.contains(&acl.name) {
                match acl_storage.delete_acl(acl.clone()).await {
                    Ok(()) => {
                        deleted.insert(acl.name.clone());
                    }
                    Err(e) => {
                        warn!("Kafka DeleteAcls failed to delete ACL {}: {}", acl.name, e);
                        error_code = ResponseError::UnknownServerError.code();
                        error_message = Some(StrBytes::from(e.to_string()));
                    }
                }
            }

            matching.push(
                DeleteAclsMatchingAcl::default()
                    .with_error_code(error_code)
                    .with_error_message(error_message)
                    .with_resource_type(b.resource_type)
                    .with_resource_name(StrBytes::from(b.resource_name))
                    .with_pattern_type(b.pattern_type)
                    .with_principal(StrBytes::from(b.principal))
                    .with_host(StrBytes::from(b.host))
                    .with_operation(b.operation)
                    .with_permission_type(b.permission_type),
            );
        }
        filter_results.push(
            DeleteAclsFilterResult::default()
                .with_error_code(0)
                .with_matching_acls(matching),
        );
    }

    Some(KafkaPacket::DeleteAclsResponse(
        DeleteAclsResponse::default().with_filter_results(filter_results),
    ))
}
