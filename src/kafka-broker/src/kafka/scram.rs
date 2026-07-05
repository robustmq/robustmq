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

use crate::handler::tenant::get_tenant;
use common_config::broker::broker_config;
use grpc_clients::meta::kafka::call::{
    delete_scram_credential, list_scram_credential, set_scram_credential,
};
use hmac::{Hmac, Mac};
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::alter_user_scram_credentials_request::{
    ScramCredentialDeletion, ScramCredentialUpsertion,
};
use kafka_protocol::messages::alter_user_scram_credentials_response::AlterUserScramCredentialsResult;
use kafka_protocol::messages::describe_user_scram_credentials_response::{
    CredentialInfo, DescribeUserScramCredentialsResult,
};
use kafka_protocol::messages::{
    AlterUserScramCredentialsRequest, AlterUserScramCredentialsResponse,
    DescribeUserScramCredentialsRequest, DescribeUserScramCredentialsResponse,
};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::kafka::scram::{
    KafkaScramCredential, SCRAM_MECHANISM_SHA_256, SCRAM_MECHANISM_SHA_512, SCRAM_MIN_ITERATIONS,
};
use protocol::kafka::packet::KafkaPacket;
use protocol::meta::meta_service_kafka::{
    DeleteScramCredentialRequest, ListScramCredentialRequest, SetScramCredentialRequest,
};
use sha2::{Digest, Sha256, Sha512};
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

// RFC 5802: ClientKey = HMAC(SaltedPassword, "Client Key"); StoredKey = H(ClientKey);
// ServerKey = HMAC(SaltedPassword, "Server Key"). Only the derived keys are kept;
// the salted password is discarded after this call.
fn derive_keys(mechanism: i8, salted_password: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    match mechanism {
        SCRAM_MECHANISM_SHA_256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(salted_password).ok()?;
            mac.update(b"Client Key");
            let client_key = mac.finalize().into_bytes();
            let stored_key = Sha256::digest(client_key).to_vec();
            let mut mac = Hmac::<Sha256>::new_from_slice(salted_password).ok()?;
            mac.update(b"Server Key");
            Some((stored_key, mac.finalize().into_bytes().to_vec()))
        }
        SCRAM_MECHANISM_SHA_512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(salted_password).ok()?;
            mac.update(b"Client Key");
            let client_key = mac.finalize().into_bytes();
            let stored_key = Sha512::digest(client_key).to_vec();
            let mut mac = Hmac::<Sha512>::new_from_slice(salted_password).ok()?;
            mac.update(b"Server Key");
            Some((stored_key, mac.finalize().into_bytes().to_vec()))
        }
        _ => None,
    }
}

fn validate_upsertion(upsertion: &ScramCredentialUpsertion) -> Result<(), (i16, String)> {
    if upsertion.name.is_empty() {
        return Err((
            ResponseError::InvalidRequest.code(),
            "user name must not be empty".to_string(),
        ));
    }
    if upsertion.mechanism != SCRAM_MECHANISM_SHA_256
        && upsertion.mechanism != SCRAM_MECHANISM_SHA_512
    {
        return Err((
            ResponseError::UnsupportedSaslMechanism.code(),
            format!("unsupported SCRAM mechanism: {}", upsertion.mechanism),
        ));
    }
    if upsertion.iterations < SCRAM_MIN_ITERATIONS {
        return Err((
            ResponseError::UnacceptableCredential.code(),
            format!("iterations must be at least {}", SCRAM_MIN_ITERATIONS),
        ));
    }
    if upsertion.salt.is_empty() || upsertion.salted_password.is_empty() {
        return Err((
            ResponseError::UnacceptableCredential.code(),
            "salt and salted password must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_deletion(deletion: &ScramCredentialDeletion) -> Result<(), (i16, String)> {
    if deletion.name.is_empty() {
        return Err((
            ResponseError::InvalidRequest.code(),
            "user name must not be empty".to_string(),
        ));
    }
    if deletion.mechanism != SCRAM_MECHANISM_SHA_256
        && deletion.mechanism != SCRAM_MECHANISM_SHA_512
    {
        return Err((
            ResponseError::UnsupportedSaslMechanism.code(),
            format!("unsupported SCRAM mechanism: {}", deletion.mechanism),
        ));
    }
    Ok(())
}

// Users whose (user, mechanism) pairs appear more than once across the whole
// request fail with DUPLICATE_RESOURCE, per KIP-554.
fn duplicate_users(req: &AlterUserScramCredentialsRequest) -> HashSet<String> {
    let mut seen = HashSet::new();
    let mut duplicated = HashSet::new();
    let pairs = req
        .deletions
        .iter()
        .map(|d| (d.name.to_string(), d.mechanism))
        .chain(
            req.upsertions
                .iter()
                .map(|u| (u.name.to_string(), u.mechanism)),
        );
    for (user, mechanism) in pairs {
        if !seen.insert((user.clone(), mechanism)) {
            duplicated.insert(user);
        }
    }
    duplicated
}

async fn list_credentials(
    sdm: &Arc<StorageDriverManager>,
) -> Result<Vec<KafkaScramCredential>, String> {
    let client_pool = &sdm.engine_storage_handler.client_pool;
    let addrs = broker_config().get_meta_service_addr();
    let reply = list_scram_credential(
        client_pool,
        &addrs,
        ListScramCredentialRequest {
            tenant: get_tenant().to_string(),
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut credentials = Vec::with_capacity(reply.credentials.len());
    for raw in reply.credentials {
        credentials.push(KafkaScramCredential::decode(&raw).map_err(|e| e.to_string())?);
    }
    Ok(credentials)
}

pub async fn process_alter_user_scram_credentials(
    sdm: &Arc<StorageDriverManager>,
    req: &AlterUserScramCredentialsRequest,
) -> Option<KafkaPacket> {
    let client_pool = sdm.engine_storage_handler.client_pool.clone();
    let addrs = broker_config().get_meta_service_addr();

    let stored = match list_credentials(sdm).await {
        Ok(credentials) => credentials,
        Err(e) => {
            warn!("Kafka AlterUserScramCredentials failed to list: {}", e);
            let users: BTreeMap<String, (i16, Option<String>)> = req
                .deletions
                .iter()
                .map(|d| d.name.to_string())
                .chain(req.upsertions.iter().map(|u| u.name.to_string()))
                .map(|u| {
                    (
                        u,
                        (ResponseError::UnknownServerError.code(), Some(e.clone())),
                    )
                })
                .collect();
            return Some(alter_response(users));
        }
    };

    // First error per user wins; later ops for an already-failed user are skipped.
    let mut results: BTreeMap<String, (i16, Option<String>)> = BTreeMap::new();
    for user in duplicate_users(req) {
        results.insert(
            user,
            (
                ResponseError::DuplicateResource.code(),
                Some("duplicate (user, mechanism) in request".to_string()),
            ),
        );
    }

    for deletion in &req.deletions {
        let user = deletion.name.to_string();
        if results.get(&user).is_some_and(|(code, _)| *code != 0) {
            continue;
        }
        let outcome = match validate_deletion(deletion) {
            Ok(()) => {
                let exists = stored
                    .iter()
                    .any(|c| c.user == user && c.mechanism == deletion.mechanism);
                if !exists {
                    Err((
                        ResponseError::ResourceNotFound.code(),
                        "no such SCRAM credential".to_string(),
                    ))
                } else {
                    delete_scram_credential(
                        &client_pool,
                        &addrs,
                        DeleteScramCredentialRequest {
                            tenant: get_tenant().to_string(),
                            user: user.clone(),
                            mechanism: deletion.mechanism as i32,
                        },
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| (ResponseError::UnknownServerError.code(), e.to_string()))
                }
            }
            Err(e) => Err(e),
        };
        record_outcome(&mut results, user, outcome);
    }

    for upsertion in &req.upsertions {
        let user = upsertion.name.to_string();
        if results.get(&user).is_some_and(|(code, _)| *code != 0) {
            continue;
        }
        let outcome = match validate_upsertion(upsertion) {
            Ok(()) => match derive_keys(upsertion.mechanism, &upsertion.salted_password) {
                Some((stored_key, server_key)) => {
                    let credential = KafkaScramCredential {
                        tenant: get_tenant().to_string(),
                        user: user.clone(),
                        mechanism: upsertion.mechanism,
                        iterations: upsertion.iterations,
                        salt: upsertion.salt.to_vec(),
                        stored_key,
                        server_key,
                    };
                    match credential.encode() {
                        Ok(encoded) => set_scram_credential(
                            &client_pool,
                            &addrs,
                            SetScramCredentialRequest {
                                credential: encoded,
                            },
                        )
                        .await
                        .map(|_| ())
                        .map_err(|e| (ResponseError::UnknownServerError.code(), e.to_string())),
                        Err(e) => Err((ResponseError::UnknownServerError.code(), e.to_string())),
                    }
                }
                None => Err((
                    ResponseError::UnsupportedSaslMechanism.code(),
                    "failed to derive SCRAM keys".to_string(),
                )),
            },
            Err(e) => Err(e),
        };
        record_outcome(&mut results, user, outcome);
    }

    Some(alter_response(results))
}

fn record_outcome(
    results: &mut BTreeMap<String, (i16, Option<String>)>,
    user: String,
    outcome: Result<(), (i16, String)>,
) {
    match outcome {
        Ok(()) => {
            results.entry(user).or_insert((0, None));
        }
        Err((code, message)) => {
            results.insert(user, (code, Some(message)));
        }
    }
}

fn alter_response(results: BTreeMap<String, (i16, Option<String>)>) -> KafkaPacket {
    let results = results
        .into_iter()
        .map(|(user, (code, message))| {
            AlterUserScramCredentialsResult::default()
                .with_user(StrBytes::from(user))
                .with_error_code(code)
                .with_error_message(message.map(StrBytes::from))
        })
        .collect();
    KafkaPacket::AlterUserScramCredentialsResponse(
        AlterUserScramCredentialsResponse::default().with_results(results),
    )
}

pub async fn process_describe_user_scram_credentials(
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeUserScramCredentialsRequest,
) -> Option<KafkaPacket> {
    let stored = match list_credentials(sdm).await {
        Ok(credentials) => credentials,
        Err(e) => {
            warn!("Kafka DescribeUserScramCredentials failed to list: {}", e);
            return Some(KafkaPacket::DescribeUserScramCredentialsResponse(
                DescribeUserScramCredentialsResponse::default()
                    .with_error_code(ResponseError::UnknownServerError.code())
                    .with_error_message(Some(StrBytes::from(e))),
            ));
        }
    };

    // Only mechanism + iterations are ever exposed; keys and salt stay server-side.
    let mut by_user: BTreeMap<String, Vec<CredentialInfo>> = BTreeMap::new();
    for credential in &stored {
        by_user.entry(credential.user.clone()).or_default().push(
            CredentialInfo::default()
                .with_mechanism(credential.mechanism)
                .with_iterations(credential.iterations),
        );
    }

    let results = match &req.users {
        None => by_user
            .into_iter()
            .map(|(user, infos)| user_result(user, 0, None, infos))
            .collect(),
        Some(users) if users.is_empty() => by_user
            .into_iter()
            .map(|(user, infos)| user_result(user, 0, None, infos))
            .collect(),
        Some(users) => {
            let mut seen = HashSet::new();
            let mut duplicated = HashSet::new();
            for u in users {
                if !seen.insert(u.name.to_string()) {
                    duplicated.insert(u.name.to_string());
                }
            }
            seen.into_iter()
                .collect::<std::collections::BTreeSet<String>>()
                .into_iter()
                .map(|user| {
                    if duplicated.contains(&user) {
                        return user_result(
                            user,
                            ResponseError::DuplicateResource.code(),
                            Some("user requested more than once".to_string()),
                            vec![],
                        );
                    }
                    match by_user.remove(&user) {
                        Some(infos) => user_result(user, 0, None, infos),
                        None => user_result(
                            user,
                            ResponseError::ResourceNotFound.code(),
                            Some("no SCRAM credentials for this user".to_string()),
                            vec![],
                        ),
                    }
                })
                .collect()
        }
    };

    Some(KafkaPacket::DescribeUserScramCredentialsResponse(
        DescribeUserScramCredentialsResponse::default()
            .with_error_code(0)
            .with_results(results),
    ))
}

fn user_result(
    user: String,
    code: i16,
    message: Option<String>,
    infos: Vec<CredentialInfo>,
) -> DescribeUserScramCredentialsResult {
    let mut infos = infos;
    infos.sort_by_key(|i| i.mechanism);
    DescribeUserScramCredentialsResult::default()
        .with_user(StrBytes::from(user))
        .with_error_code(code)
        .with_error_message(message.map(StrBytes::from))
        .with_credential_infos(infos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn derive_keys_matches_rfc5802_construction() {
        let salted_password = b"pencil-salted";
        let (stored_key, server_key) =
            derive_keys(SCRAM_MECHANISM_SHA_256, salted_password).unwrap();

        let mut mac = Hmac::<Sha256>::new_from_slice(salted_password).unwrap();
        mac.update(b"Client Key");
        let expected_stored = Sha256::digest(mac.finalize().into_bytes()).to_vec();
        assert_eq!(stored_key, expected_stored);
        assert_eq!(stored_key.len(), 32);
        assert_eq!(server_key.len(), 32);
        assert_ne!(stored_key, server_key);

        let (stored_512, server_512) =
            derive_keys(SCRAM_MECHANISM_SHA_512, salted_password).unwrap();
        assert_eq!(stored_512.len(), 64);
        assert_eq!(server_512.len(), 64);

        assert!(derive_keys(0, salted_password).is_none());
    }

    fn upsertion(mechanism: i8, iterations: i32) -> ScramCredentialUpsertion {
        ScramCredentialUpsertion::default()
            .with_name(StrBytes::from_static_str("alice"))
            .with_mechanism(mechanism)
            .with_iterations(iterations)
            .with_salt(Bytes::from_static(b"salt"))
            .with_salted_password(Bytes::from_static(b"salted"))
    }

    #[test]
    fn upsertion_validation() {
        assert!(validate_upsertion(&upsertion(SCRAM_MECHANISM_SHA_256, 4096)).is_ok());
        assert_eq!(
            validate_upsertion(&upsertion(SCRAM_MECHANISM_SHA_256, 4095))
                .unwrap_err()
                .0,
            ResponseError::UnacceptableCredential.code()
        );
        assert_eq!(
            validate_upsertion(&upsertion(3, 4096)).unwrap_err().0,
            ResponseError::UnsupportedSaslMechanism.code()
        );
        let empty_salt = upsertion(SCRAM_MECHANISM_SHA_256, 4096).with_salt(Bytes::new());
        assert_eq!(
            validate_upsertion(&empty_salt).unwrap_err().0,
            ResponseError::UnacceptableCredential.code()
        );
    }

    #[test]
    fn duplicate_pairs_are_detected_across_deletions_and_upsertions() {
        let req = AlterUserScramCredentialsRequest::default()
            .with_deletions(vec![ScramCredentialDeletion::default()
                .with_name(StrBytes::from_static_str("alice"))
                .with_mechanism(SCRAM_MECHANISM_SHA_256)])
            .with_upsertions(vec![
                upsertion(SCRAM_MECHANISM_SHA_256, 4096),
                upsertion(SCRAM_MECHANISM_SHA_512, 4096)
                    .with_name(StrBytes::from_static_str("bob")),
            ]);
        let dup = duplicate_users(&req);
        assert!(dup.contains("alice"));
        assert!(!dup.contains("bob"));
    }
}
