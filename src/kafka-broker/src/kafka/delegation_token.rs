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

//! Delegation token (KIP-48) metadata management: create/renew/expire/describe.
//!
//! This is metadata management only. Nothing here (or anywhere else in the
//! broker) verifies a token's `hmac` during authentication — SASL support
//! for delegation-token re-auth is a separate, not-yet-implemented effort.
//! Because of that, every owner/requester/renewer permission check that real
//! Kafka performs against the authenticated principal is skipped: every
//! request is attributed to a fixed placeholder identity, and Renew/Expire
//! don't check whether the caller is actually the owner or a renewer.

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::handler::tenant::get_tenant;
use common_base::tools::now_millis;
use common_base::uuid::unique_id;
use common_config::broker::broker_config;
use grpc_clients::meta::common::call::{get_resource_config, set_resource_config};
use grpc_clients::meta::kafka::call::{
    delete_kafka_delegation_token, list_kafka_delegation_token, set_kafka_delegation_token,
};
use grpc_clients::pool::ClientPool;
use hmac::{Hmac, Mac};
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::describe_delegation_token_response::{
    DescribedDelegationToken, DescribedDelegationTokenRenewer,
};
use kafka_protocol::messages::{
    CreateDelegationTokenRequest, CreateDelegationTokenResponse, DescribeDelegationTokenRequest,
    DescribeDelegationTokenResponse, ExpireDelegationTokenRequest, ExpireDelegationTokenResponse,
    RenewDelegationTokenRequest, RenewDelegationTokenResponse,
};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::kafka::delegation_token::{KafkaDelegationToken, KafkaTokenPrincipal};
use protocol::kafka::packet::KafkaPacket;
use protocol::meta::meta_service_common::{GetResourceConfigRequest, SetResourceConfigRequest};
use protocol::meta::meta_service_kafka::{
    DeleteKafkaDelegationTokenRequest, ListKafkaDelegationTokenRequest,
    SetKafkaDelegationTokenRequest,
};
use rand::RngCore;
use sha2::Sha256;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

type HmacSha256 = Hmac<Sha256>;

// Sweep interval for the background task that deletes tokens past their
// hard `max_timestamp_ms` (see `reap_expired_tokens`). A token that's merely
// past its soft `expiry_timestamp_ms` is left alone — Renew can still revive
// it — only `max_timestamp_ms`, which nothing can ever extend, makes a token
// permanently dead and safe to delete.
const REAPER_INTERVAL_MS: u64 = 60_000;
static REAPER_STARTED: OnceLock<()> = OnceLock::new();

// No real authentication exists yet (see module doc), so every request is
// attributed to this fixed placeholder rather than a real principal.
const ANONYMOUS_PRINCIPAL_TYPE: &str = "User";
const ANONYMOUS_PRINCIPAL_NAME: &str = "ANONYMOUS";

// Default/max token lifetime when a client passes -1, matching Kafka's own
// default (`delegation.token.max.lifetime.ms`).
const DEFAULT_TOKEN_LIFETIME_MS: i64 = 7 * 24 * 60 * 60 * 1000;

pub async fn process_create_delegation_token(
    sdm: &Arc<StorageDriverManager>,
    req: &CreateDelegationTokenRequest,
) -> Option<KafkaPacket> {
    ensure_reaper_started(sdm);

    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let secret = match get_or_create_secret_key(client_pool, &addrs).await {
        Ok(s) => s,
        Err(e) => {
            warn!(
                "Kafka CreateDelegationToken failed to get the signing key: {}",
                e
            );
            return Some(create_error_response(
                ResponseError::UnknownServerError.code(),
            ));
        }
    };

    let token_id = unique_id();
    let hmac = match compute_hmac(&secret, &token_id) {
        Ok(h) => h,
        Err(e) => {
            warn!("Kafka CreateDelegationToken failed to compute hmac: {}", e);
            return Some(create_error_response(
                ResponseError::UnknownServerError.code(),
            ));
        }
    };

    let issue_timestamp_ms = now_millis() as i64;
    let max_lifetime_ms = if req.max_lifetime_ms > 0 {
        req.max_lifetime_ms
    } else {
        DEFAULT_TOKEN_LIFETIME_MS
    };
    let max_timestamp_ms = issue_timestamp_ms.saturating_add(max_lifetime_ms);

    let requester = KafkaTokenPrincipal {
        principal_type: ANONYMOUS_PRINCIPAL_TYPE.to_string(),
        principal_name: ANONYMOUS_PRINCIPAL_NAME.to_string(),
    };
    let owner = match (&req.owner_principal_type, &req.owner_principal_name) {
        (Some(principal_type), Some(principal_name)) => KafkaTokenPrincipal {
            principal_type: principal_type.to_string(),
            principal_name: principal_name.to_string(),
        },
        _ => requester.clone(),
    };
    let renewers = req
        .renewers
        .iter()
        .map(|r| KafkaTokenPrincipal {
            principal_type: r.principal_type.to_string(),
            principal_name: r.principal_name.to_string(),
        })
        .collect();

    let token = KafkaDelegationToken {
        tenant: get_tenant().to_string(),
        token_id: token_id.clone(),
        hmac: hmac.clone(),
        owner: owner.clone(),
        token_requester: requester.clone(),
        renewers,
        issue_timestamp_ms,
        expiry_timestamp_ms: max_timestamp_ms,
        max_timestamp_ms,
    };

    if let Err(e) = save_token(client_pool, &addrs, &token).await {
        warn!("Kafka CreateDelegationToken storage error: {}", e);
        return Some(create_error_response(
            ResponseError::UnknownServerError.code(),
        ));
    }

    Some(KafkaPacket::CreateDelegationTokenResponse(
        CreateDelegationTokenResponse::default()
            .with_error_code(0)
            .with_principal_type(StrBytes::from(owner.principal_type))
            .with_principal_name(StrBytes::from(owner.principal_name))
            .with_token_requester_principal_type(StrBytes::from(requester.principal_type))
            .with_token_requester_principal_name(StrBytes::from(requester.principal_name))
            .with_issue_timestamp_ms(issue_timestamp_ms)
            .with_expiry_timestamp_ms(max_timestamp_ms)
            .with_max_timestamp_ms(max_timestamp_ms)
            .with_token_id(StrBytes::from(token_id))
            .with_hmac(bytes::Bytes::from(hmac)),
    ))
}

pub async fn process_renew_delegation_token(
    sdm: &Arc<StorageDriverManager>,
    req: &RenewDelegationTokenRequest,
) -> Option<KafkaPacket> {
    ensure_reaper_started(sdm);
    let renew_period_ms = if req.renew_period_ms > 0 {
        req.renew_period_ms
    } else {
        DEFAULT_TOKEN_LIFETIME_MS
    };

    let result = update_expiry(sdm, &req.hmac, |now, max_timestamp_ms| {
        now.saturating_add(renew_period_ms).min(max_timestamp_ms)
    })
    .await;

    match result {
        Ok(expiry_timestamp_ms) => Some(KafkaPacket::RenewDelegationTokenResponse(
            RenewDelegationTokenResponse::default()
                .with_error_code(0)
                .with_expiry_timestamp_ms(expiry_timestamp_ms),
        )),
        Err((code, message)) => {
            warn!("Kafka RenewDelegationToken failed: {}", message);
            Some(KafkaPacket::RenewDelegationTokenResponse(
                RenewDelegationTokenResponse::default().with_error_code(code),
            ))
        }
    }
}

pub async fn process_expire_delegation_token(
    sdm: &Arc<StorageDriverManager>,
    req: &ExpireDelegationTokenRequest,
) -> Option<KafkaPacket> {
    ensure_reaper_started(sdm);
    let expiry_time_period_ms = req.expiry_time_period_ms;

    let result = update_expiry(sdm, &req.hmac, |now, max_timestamp_ms| {
        if expiry_time_period_ms <= 0 {
            now
        } else {
            now.saturating_add(expiry_time_period_ms)
                .min(max_timestamp_ms)
        }
    })
    .await;

    match result {
        Ok(expiry_timestamp_ms) => Some(KafkaPacket::ExpireDelegationTokenResponse(
            ExpireDelegationTokenResponse::default()
                .with_error_code(0)
                .with_expiry_timestamp_ms(expiry_timestamp_ms),
        )),
        Err((code, message)) => {
            warn!("Kafka ExpireDelegationToken failed: {}", message);
            Some(KafkaPacket::ExpireDelegationTokenResponse(
                ExpireDelegationTokenResponse::default().with_error_code(code),
            ))
        }
    }
}

pub async fn process_describe_delegation_token(
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeDelegationTokenRequest,
) -> Option<KafkaPacket> {
    ensure_reaper_started(sdm);
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let reply = match list_kafka_delegation_token(
        client_pool,
        &addrs,
        ListKafkaDelegationTokenRequest {
            tenant: get_tenant().to_string(),
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Kafka DescribeDelegationToken failed to list tokens: {}", e);
            return Some(KafkaPacket::DescribeDelegationTokenResponse(
                DescribeDelegationTokenResponse::default()
                    .with_error_code(ResponseError::UnknownServerError.code()),
            ));
        }
    };

    let mut tokens = Vec::with_capacity(reply.tokens.len());
    for raw in reply.tokens {
        let token = match KafkaDelegationToken::decode(&raw) {
            Ok(t) => t,
            Err(e) => {
                warn!(
                    "Kafka DescribeDelegationToken failed to decode a stored token: {}",
                    e
                );
                return Some(KafkaPacket::DescribeDelegationTokenResponse(
                    DescribeDelegationTokenResponse::default()
                        .with_error_code(ResponseError::UnknownServerError.code()),
                ));
            }
        };
        if owner_matches(&token, req) {
            tokens.push(to_described_token(token));
        }
    }

    Some(KafkaPacket::DescribeDelegationTokenResponse(
        DescribeDelegationTokenResponse::default()
            .with_error_code(0)
            .with_tokens(tokens),
    ))
}

fn secret_key_resource() -> Vec<String> {
    vec!["kafka".to_string(), "delegation_token_secret".to_string()]
}

// The HMAC signing key must be identical across every broker node (a client
// may reconnect to any node), so it's bootstrapped once into meta-service on
// first use rather than generated per-process. Two nodes racing to
// initialize it at the same time is a narrow, one-time startup race we
// accept rather than build distributed-lock machinery for — but we must not
// let the loser of that race sign with the key it generated locally instead
// of the one that actually won, so re-read after writing rather than trusting
// the value we just generated.
async fn get_or_create_secret_key(
    client_pool: &Arc<ClientPool>,
    addrs: &[String],
) -> Result<Vec<u8>, String> {
    let resources = secret_key_resource();
    let reply = get_resource_config(
        client_pool,
        addrs,
        GetResourceConfigRequest {
            resources: resources.clone(),
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    if !reply.config.is_empty() {
        return Ok(reply.config);
    }

    let mut key = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    set_resource_config(
        client_pool,
        addrs,
        SetResourceConfigRequest {
            resources: resources.clone(),
            config: key,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    // Re-read rather than trusting the value just generated: if another node
    // won the race to initialize this key, its value — not ours — is now the
    // canonical one every node must sign with. The read is served from a node's
    // local replica, which may briefly lag the just-committed set, so poll until
    // it converges.
    for _ in 0..30 {
        let reply = get_resource_config(
            client_pool,
            addrs,
            GetResourceConfigRequest {
                resources: resources.clone(),
            },
        )
        .await
        .map_err(|e| e.to_string())?;
        if !reply.config.is_empty() {
            return Ok(reply.config);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err("delegation token secret key vanished immediately after being set".to_string())
}

fn compute_hmac(secret: &[u8], token_id: &str) -> Result<Vec<u8>, String> {
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| e.to_string())?;
    mac.update(token_id.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

async fn find_token_by_hmac(
    client_pool: &Arc<ClientPool>,
    addrs: &[String],
    hmac: &[u8],
) -> Result<Option<KafkaDelegationToken>, String> {
    let reply = list_kafka_delegation_token(
        client_pool,
        addrs,
        ListKafkaDelegationTokenRequest {
            tenant: get_tenant().to_string(),
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    for raw in reply.tokens {
        let token = KafkaDelegationToken::decode(&raw).map_err(|e| e.to_string())?;
        if token.hmac == hmac {
            return Ok(Some(token));
        }
    }
    Ok(None)
}

async fn save_token(
    client_pool: &Arc<ClientPool>,
    addrs: &[String],
    token: &KafkaDelegationToken,
) -> Result<(), String> {
    let request = SetKafkaDelegationTokenRequest {
        token: token.encode().map_err(|e| e.to_string())?,
    };
    set_kafka_delegation_token(client_pool, addrs, request)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Every broker node runs its own copy of this loop against the same
// meta-service state; a token past `max_timestamp_ms` deleted twice (by two
// nodes racing) is harmless — `delete_kafka_delegation_token` on an
// already-gone key is a no-op — so no cross-node coordination is needed.
async fn reap_expired_tokens(sdm: &Arc<StorageDriverManager>) {
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let reply = match list_kafka_delegation_token(
        client_pool,
        &addrs,
        ListKafkaDelegationTokenRequest {
            tenant: get_tenant().to_string(),
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Kafka delegation-token reaper failed to list tokens: {}", e);
            return;
        }
    };

    let now = now_millis() as i64;
    for raw in reply.tokens {
        let token = match KafkaDelegationToken::decode(&raw) {
            Ok(t) => t,
            Err(e) => {
                warn!(
                    "Kafka delegation-token reaper failed to decode a stored token: {}",
                    e
                );
                continue;
            }
        };
        if now <= token.max_timestamp_ms {
            continue;
        }
        let request = DeleteKafkaDelegationTokenRequest {
            tenant: token.tenant.clone(),
            token_id: token.token_id.clone(),
        };
        if let Err(e) = delete_kafka_delegation_token(client_pool, &addrs, request).await {
            warn!(
                "Kafka delegation-token reaper failed to delete expired token '{}': {}",
                token.token_id, e
            );
        }
    }
}

// The reaper needs `StorageDriverManager` (to reach meta-service), which
// isn't available at module-init time, so it's lazily spawned from the
// first delegation-token request instead of at broker startup — same
// approach `GroupCoordinator::ensure_reaper_started` uses for the
// consumer-group session reaper.
fn ensure_reaper_started(sdm: &Arc<StorageDriverManager>) {
    REAPER_STARTED.get_or_init(|| {
        let sdm = sdm.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(REAPER_INTERVAL_MS)).await;
                reap_expired_tokens(&sdm).await;
            }
        });
    });
}

fn create_error_response(code: i16) -> KafkaPacket {
    KafkaPacket::CreateDelegationTokenResponse(
        CreateDelegationTokenResponse::default().with_error_code(code),
    )
}

// Renew and Expire share the same shape: find the token by the `hmac` the
// client presents (proof of possession, not `token_id`), recompute
// `expiry_timestamp_ms` capped at `max_timestamp_ms`, and write the record
// back. Renew extends it (bounded by `renew_period_ms`), Expire shortens it
// (bounded by `expiry_time_period_ms`, `<= 0` meaning "now").
async fn update_expiry(
    sdm: &Arc<StorageDriverManager>,
    hmac: &bytes::Bytes,
    new_expiry_timestamp_ms: impl FnOnce(i64, i64) -> i64,
) -> Result<i64, (i16, String)> {
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();

    let mut token = match find_token_by_hmac(client_pool, &addrs, hmac).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Err((
                ResponseError::DelegationTokenNotFound.code(),
                "no delegation token matches the given hmac".to_string(),
            ));
        }
        Err(e) => return Err((ResponseError::UnknownServerError.code(), e)),
    };

    let now = now_millis() as i64;
    // `max_timestamp_ms` is the one thing Renew/Expire can never move past —
    // once it's gone, the token is permanently dead (the reaper will delete
    // it in due course) and must be rejected outright rather than "renewed"
    // to an expiry timestamp that's already in the past.
    if now > token.max_timestamp_ms {
        return Err((
            ResponseError::DelegationTokenExpired.code(),
            "delegation token is past its max lifetime".to_string(),
        ));
    }
    token.expiry_timestamp_ms = new_expiry_timestamp_ms(now, token.max_timestamp_ms);

    save_token(client_pool, &addrs, &token)
        .await
        .map_err(|e| (ResponseError::UnknownServerError.code(), e))?;

    Ok(token.expiry_timestamp_ms)
}

// DescribeDelegationToken owner filter: `owners = None` means "every token".
// We also treat `Some([])` (an explicitly empty list) as "every token" —
// this is our own choice, not something verified against real Kafka's
// source for this exact edge case; revisit if it turns out real clients
// send an empty-but-present list to mean something else.
fn owner_matches(token: &KafkaDelegationToken, req: &DescribeDelegationTokenRequest) -> bool {
    match &req.owners {
        None => true,
        Some(owners) if owners.is_empty() => true,
        Some(owners) => owners.iter().any(|o| {
            o.principal_type.as_str() == token.owner.principal_type
                && o.principal_name.as_str() == token.owner.principal_name
        }),
    }
}

fn to_described_token(token: KafkaDelegationToken) -> DescribedDelegationToken {
    let renewers = token
        .renewers
        .into_iter()
        .map(|r| {
            DescribedDelegationTokenRenewer::default()
                .with_principal_type(StrBytes::from(r.principal_type))
                .with_principal_name(StrBytes::from(r.principal_name))
        })
        .collect();

    DescribedDelegationToken::default()
        .with_principal_type(StrBytes::from(token.owner.principal_type))
        .with_principal_name(StrBytes::from(token.owner.principal_name))
        .with_token_requester_principal_type(StrBytes::from(token.token_requester.principal_type))
        .with_token_requester_principal_name(StrBytes::from(token.token_requester.principal_name))
        .with_issue_timestamp(token.issue_timestamp_ms)
        .with_expiry_timestamp(token.expiry_timestamp_ms)
        .with_max_timestamp(token.max_timestamp_ms)
        .with_token_id(StrBytes::from(token.token_id))
        .with_hmac(bytes::Bytes::from(token.hmac))
        .with_renewers(renewers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(owner_type: &str, owner_name: &str) -> KafkaDelegationToken {
        KafkaDelegationToken {
            tenant: "default".to_string(),
            token_id: "t1".to_string(),
            hmac: vec![1, 2, 3],
            owner: KafkaTokenPrincipal {
                principal_type: owner_type.to_string(),
                principal_name: owner_name.to_string(),
            },
            token_requester: KafkaTokenPrincipal {
                principal_type: owner_type.to_string(),
                principal_name: owner_name.to_string(),
            },
            renewers: vec![],
            issue_timestamp_ms: 0,
            expiry_timestamp_ms: 1000,
            max_timestamp_ms: 2000,
        }
    }

    fn describe_req(owners: Option<Vec<(&str, &str)>>) -> DescribeDelegationTokenRequest {
        DescribeDelegationTokenRequest::default().with_owners(owners.map(|list| {
            list.into_iter()
                .map(|(t, n)| {
                    kafka_protocol::messages::describe_delegation_token_request::DescribeDelegationTokenOwner::default()
                        .with_principal_type(StrBytes::from(t.to_string()))
                        .with_principal_name(StrBytes::from(n.to_string()))
                })
                .collect()
        }))
    }

    #[test]
    fn owner_matches_none_and_empty_mean_everything() {
        let t = token("User", "alice");
        assert!(owner_matches(&t, &describe_req(None)));
        assert!(owner_matches(&t, &describe_req(Some(vec![]))));
    }

    #[test]
    fn owner_matches_filters_by_exact_principal() {
        let t = token("User", "alice");
        assert!(owner_matches(
            &t,
            &describe_req(Some(vec![("User", "alice")]))
        ));
        assert!(!owner_matches(
            &t,
            &describe_req(Some(vec![("User", "bob")]))
        ));
    }

    #[test]
    fn compute_hmac_is_deterministic_for_same_inputs() {
        let secret = b"secret-key".to_vec();
        let a = compute_hmac(&secret, "token-1").unwrap();
        let b = compute_hmac(&secret, "token-1").unwrap();
        let c = compute_hmac(&secret, "token-2").unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
