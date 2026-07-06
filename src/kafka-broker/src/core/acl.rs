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

use crate::handler::tenant::get_tenant;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::create_acls_request::AclCreation;
use kafka_protocol::protocol::StrBytes;
use metadata_struct::auth::acl::{
    EnumAclAction, EnumAclPermission, EnumAclResourceType, SecurityAcl,
};

// Kafka wire enum values (org.apache.kafka.common.{resource,acl}).
const RESOURCE_TYPE_ANY: i8 = 1;
const RESOURCE_TYPE_TOPIC: i8 = 2;
const PATTERN_TYPE_ANY: i8 = 1;
const PATTERN_TYPE_MATCH: i8 = 2;
const PATTERN_TYPE_LITERAL: i8 = 3;
const ACL_OP_ANY: i8 = 1;
const ACL_OP_ALL: i8 = 2;
const ACL_OP_READ: i8 = 3;
const ACL_OP_WRITE: i8 = 4;
const ACL_PERM_ANY: i8 = 1;
const ACL_PERM_DENY: i8 = 2;
const ACL_PERM_ALLOW: i8 = 3;

// Prefix a field with its byte length so a sequence of framed fields is injective
// regardless of what characters (including the ':' separator) the fields contain.
fn framed(s: &str) -> String {
    format!("{}:{}", s.len(), s)
}

// Map one Kafka AclCreation onto a RobustMQ SecurityAcl. The two models line up as
// "principal → operation → resource → host → effect":
//   principal "User:alice"  -> resource_type=User + resource_name="alice"
//   resource_name (a topic) -> topic
//   host                    -> ip  (empty host means "any", i.e. "*")
//   operation Read/Write/All-> Subscribe/Publish/All
//   permission_type         -> Allow/Deny
// Only literal Topic resources and Read/Write/All operations are representable today;
// anything else is rejected with a per-binding error rather than stored lossily.
pub(crate) fn to_security_acl(creation: &AclCreation) -> Result<SecurityAcl, ResponseError> {
    if creation.resource_type != RESOURCE_TYPE_TOPIC {
        return Err(ResponseError::InvalidRequest);
    }
    if creation.resource_pattern_type != PATTERN_TYPE_LITERAL {
        return Err(ResponseError::InvalidRequest);
    }

    let principal = creation.principal.to_string();
    let (principal_type, principal_name) = principal
        .split_once(':')
        .ok_or(ResponseError::InvalidPrincipalType)?;
    let resource_type = match principal_type {
        "User" => EnumAclResourceType::User,
        "ClientId" => EnumAclResourceType::ClientId,
        _ => return Err(ResponseError::InvalidPrincipalType),
    };
    if principal_name.is_empty() {
        return Err(ResponseError::InvalidPrincipalType);
    }

    let action = match creation.operation {
        ACL_OP_ALL => EnumAclAction::All,
        ACL_OP_READ => EnumAclAction::Subscribe,
        ACL_OP_WRITE => EnumAclAction::Publish,
        _ => return Err(ResponseError::InvalidRequest),
    };
    let permission = match creation.permission_type {
        ACL_PERM_ALLOW => EnumAclPermission::Allow,
        ACL_PERM_DENY => EnumAclPermission::Deny,
        _ => return Err(ResponseError::InvalidRequest),
    };

    let topic = creation.resource_name.to_string();
    if topic.is_empty() {
        return Err(ResponseError::InvalidRequest);
    }

    let host = creation.host.to_string();
    let ip = if host.is_empty() {
        "*".to_string()
    } else {
        host
    };

    // Deterministic name so re-creating the same binding is idempotent (same key).
    // Enum fields are colon-free fixed vocab; the free-text fields are length-framed
    // because principal names and IPv6 hosts may contain ':', which would otherwise
    // let distinct bindings collide into one name (and DeleteAcls deletes by name).
    let name = format!(
        "kafka:{}:{}:{}:{}:{}:{}",
        resource_type,
        action,
        permission,
        framed(principal_name),
        framed(&topic),
        framed(&ip),
    );

    Ok(SecurityAcl {
        name,
        desc: String::new(),
        tenant: get_tenant().to_string(),
        resource_type,
        resource_name: principal_name.to_string(),
        topic,
        ip,
        action,
        permission,
    })
}

// A SecurityAcl rendered back into Kafka AclBinding fields, so it can be matched
// against Describe/Delete filters and echoed in responses.
pub(crate) struct AclBinding {
    pub resource_type: i8,
    pub resource_name: String,
    pub pattern_type: i8,
    pub principal: String,
    pub host: String,
    pub operation: i8,
    pub permission_type: i8,
}

fn action_to_operation(action: EnumAclAction) -> Option<i8> {
    match action {
        EnumAclAction::All => Some(ACL_OP_ALL),
        EnumAclAction::Subscribe => Some(ACL_OP_READ),
        EnumAclAction::Publish => Some(ACL_OP_WRITE),
        // PubSub/Retain/Qos have no Kafka operation equivalent; such ACLs are not
        // representable as Kafka bindings and are omitted from Describe/Delete.
        _ => None,
    }
}

// Inverse of `to_security_acl`. Every stored ACL is a literal Topic ACL whose
// principal is the (resource_type, resource_name) pair. Returns None when the ACL
// cannot be represented as a Kafka binding (its action has no Kafka operation).
pub(crate) fn security_acl_to_binding(acl: &SecurityAcl) -> Option<AclBinding> {
    Some(AclBinding {
        resource_type: RESOURCE_TYPE_TOPIC,
        resource_name: acl.topic.clone(),
        pattern_type: PATTERN_TYPE_LITERAL,
        principal: format!("{}:{}", acl.resource_type, acl.resource_name),
        host: acl.ip.clone(),
        operation: action_to_operation(acl.action)?,
        permission_type: match acl.permission {
            EnumAclPermission::Allow => ACL_PERM_ALLOW,
            EnumAclPermission::Deny => ACL_PERM_DENY,
        },
    })
}

// Kafka filter semantics: enum fields match anything when set to ANY; string
// fields match anything when null, otherwise exact match (MATCH/wildcard is TODO).
#[allow(clippy::too_many_arguments)]
pub(crate) fn binding_matches(
    b: &AclBinding,
    resource_type_filter: i8,
    resource_name_filter: Option<&StrBytes>,
    pattern_type_filter: i8,
    principal_filter: Option<&StrBytes>,
    host_filter: Option<&StrBytes>,
    operation: i8,
    permission_type: i8,
) -> bool {
    if resource_type_filter != RESOURCE_TYPE_ANY && resource_type_filter != b.resource_type {
        return false;
    }
    if pattern_type_filter != PATTERN_TYPE_ANY
        && pattern_type_filter != PATTERN_TYPE_MATCH
        && pattern_type_filter != b.pattern_type
    {
        return false;
    }
    if let Some(f) = resource_name_filter {
        if f.as_str() != b.resource_name {
            return false;
        }
    }
    if let Some(f) = principal_filter {
        if f.as_str() != b.principal {
            return false;
        }
    }
    if let Some(f) = host_filter {
        if f.as_str() != b.host {
            return false;
        }
    }
    if operation != ACL_OP_ANY && operation != b.operation {
        return false;
    }
    if permission_type != ACL_PERM_ANY && permission_type != b.permission_type {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creation(
        resource_type: i8,
        pattern: i8,
        principal: &str,
        host: &str,
        resource: &str,
        op: i8,
        perm: i8,
    ) -> AclCreation {
        AclCreation::default()
            .with_resource_type(resource_type)
            .with_resource_pattern_type(pattern)
            .with_principal(StrBytes::from(principal.to_string()))
            .with_host(StrBytes::from(host.to_string()))
            .with_resource_name(StrBytes::from(resource.to_string()))
            .with_operation(op)
            .with_permission_type(perm)
    }

    #[test]
    fn maps_topic_user_write_allow() {
        let c = creation(
            RESOURCE_TYPE_TOPIC,
            PATTERN_TYPE_LITERAL,
            "User:alice",
            "10.0.0.1",
            "orders",
            ACL_OP_WRITE,
            ACL_PERM_ALLOW,
        );
        let acl = to_security_acl(&c).unwrap();
        assert_eq!(acl.resource_type, EnumAclResourceType::User);
        assert_eq!(acl.resource_name, "alice");
        assert_eq!(acl.topic, "orders");
        assert_eq!(acl.ip, "10.0.0.1");
        assert_eq!(acl.action, EnumAclAction::Publish);
        assert_eq!(acl.permission, EnumAclPermission::Allow);
    }

    #[test]
    fn empty_host_becomes_wildcard() {
        let c = creation(
            RESOURCE_TYPE_TOPIC,
            PATTERN_TYPE_LITERAL,
            "User:bob",
            "",
            "t",
            ACL_OP_READ,
            ACL_PERM_DENY,
        );
        let acl = to_security_acl(&c).unwrap();
        assert_eq!(acl.ip, "*");
        assert_eq!(acl.action, EnumAclAction::Subscribe);
        assert_eq!(acl.permission, EnumAclPermission::Deny);
    }

    #[test]
    fn to_security_acl_rejects_unrepresentable_bindings() {
        // (label, creation, expected error) — inputs that can't map to a SecurityAcl.
        let cases = [
            (
                "non-topic resource (Group=3)",
                creation(
                    3,
                    PATTERN_TYPE_LITERAL,
                    "User:a",
                    "*",
                    "g",
                    ACL_OP_READ,
                    ACL_PERM_ALLOW,
                ),
                ResponseError::InvalidRequest,
            ),
            (
                "prefixed pattern (4)",
                creation(
                    RESOURCE_TYPE_TOPIC,
                    4,
                    "User:a",
                    "*",
                    "t",
                    ACL_OP_READ,
                    ACL_PERM_ALLOW,
                ),
                ResponseError::InvalidRequest,
            ),
            (
                "principal without a type prefix",
                creation(
                    RESOURCE_TYPE_TOPIC,
                    PATTERN_TYPE_LITERAL,
                    "no-colon",
                    "*",
                    "t",
                    ACL_OP_READ,
                    ACL_PERM_ALLOW,
                ),
                ResponseError::InvalidPrincipalType,
            ),
            (
                "unsupported operation (Create=5)",
                creation(
                    RESOURCE_TYPE_TOPIC,
                    PATTERN_TYPE_LITERAL,
                    "User:a",
                    "*",
                    "t",
                    5,
                    ACL_PERM_ALLOW,
                ),
                ResponseError::InvalidRequest,
            ),
        ];
        for (label, c, expected) in cases {
            assert_eq!(to_security_acl(&c).unwrap_err(), expected, "{label}");
        }
    }

    #[test]
    fn colliding_bindings_get_distinct_names() {
        // Under a naive ':'-join these two distinct bindings share a name:
        //   A: principal "User:x", topic "t", host "u:v" (IPv6-like)
        //   B: principal "User:x:t", topic "u", host "v"
        // Length-framing must keep their names distinct.
        let a = to_security_acl(&creation(
            RESOURCE_TYPE_TOPIC,
            PATTERN_TYPE_LITERAL,
            "User:x",
            "u:v",
            "t",
            ACL_OP_READ,
            ACL_PERM_ALLOW,
        ))
        .unwrap();
        let b = to_security_acl(&creation(
            RESOURCE_TYPE_TOPIC,
            PATTERN_TYPE_LITERAL,
            "User:x:t",
            "v",
            "u",
            ACL_OP_READ,
            ACL_PERM_ALLOW,
        ))
        .unwrap();
        assert_ne!(a.name, b.name);
    }

    fn sample_acl() -> SecurityAcl {
        SecurityAcl {
            name: "n".to_string(),
            desc: String::new(),
            tenant: "default".to_string(),
            resource_type: EnumAclResourceType::User,
            resource_name: "alice".to_string(),
            topic: "orders".to_string(),
            ip: "10.0.0.1".to_string(),
            action: EnumAclAction::Publish,
            permission: EnumAclPermission::Allow,
        }
    }

    #[test]
    fn reverse_maps_to_kafka_binding() {
        let b = security_acl_to_binding(&sample_acl()).unwrap();
        assert_eq!(b.resource_type, RESOURCE_TYPE_TOPIC);
        assert_eq!(b.resource_name, "orders");
        assert_eq!(b.pattern_type, PATTERN_TYPE_LITERAL);
        assert_eq!(b.principal, "User:alice");
        assert_eq!(b.host, "10.0.0.1");
        assert_eq!(b.operation, ACL_OP_WRITE);
        assert_eq!(b.permission_type, ACL_PERM_ALLOW);
    }

    #[test]
    fn mqtt_only_action_is_skipped() {
        let mut acl = sample_acl();
        acl.action = EnumAclAction::Retain;
        assert!(security_acl_to_binding(&acl).is_none());
    }

    #[test]
    fn binding_matches_respects_filters() {
        let sb = StrBytes::from_static_str;
        let b = security_acl_to_binding(&sample_acl()).unwrap();

        // all-ANY / null filter matches everything
        assert!(binding_matches(
            &b,
            RESOURCE_TYPE_ANY,
            None,
            PATTERN_TYPE_ANY,
            None,
            None,
            ACL_OP_ANY,
            ACL_PERM_ANY,
        ));

        // exact match on every field
        assert!(binding_matches(
            &b,
            RESOURCE_TYPE_TOPIC,
            Some(&sb("orders")),
            PATTERN_TYPE_LITERAL,
            Some(&sb("User:alice")),
            Some(&sb("10.0.0.1")),
            ACL_OP_WRITE,
            ACL_PERM_ALLOW,
        ));

        // any single mismatching field breaks the match
        assert!(!binding_matches(
            &b,
            RESOURCE_TYPE_ANY,
            None,
            PATTERN_TYPE_ANY,
            Some(&sb("User:bob")),
            None,
            ACL_OP_ANY,
            ACL_PERM_ANY,
        )); // principal
        assert!(!binding_matches(
            &b,
            RESOURCE_TYPE_ANY,
            None,
            PATTERN_TYPE_ANY,
            None,
            None,
            ACL_OP_READ,
            ACL_PERM_ANY,
        )); // operation
        assert!(!binding_matches(
            &b,
            RESOURCE_TYPE_ANY,
            None,
            4,
            None,
            None,
            ACL_OP_ANY,
            ACL_PERM_ANY,
        )); // prefixed pattern must not match a literal binding
    }
}
