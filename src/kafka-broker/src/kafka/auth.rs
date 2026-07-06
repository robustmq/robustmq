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

use std::sync::Arc;

use bytes::Bytes;
use common_config::broker::broker_config;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::{
    SaslAuthenticateRequest, SaslAuthenticateResponse, SaslHandshakeRequest, SaslHandshakeResponse,
};
use kafka_protocol::protocol::StrBytes;
use protocol::kafka::packet::KafkaPacket;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tracing::warn;

use metadata_struct::kafka::scram::KafkaScramCredential;

use crate::core::cache::KafkaCacheManager;
use crate::core::sasl::{
    build_server_first, mechanism_to_code, parse_client_first, verify_client_final, SaslSession,
};

const SERVER_NONCE_LEN: usize = 24;

pub fn process_sasl_handshake(
    cache: &Arc<KafkaCacheManager>,
    connection_id: u64,
    req: &SaslHandshakeRequest,
) -> Option<KafkaPacket> {
    let mechanisms = configured_mechanisms();
    let requested = req.mechanism.to_string();

    // The mechanism must be both configured and one we actually implement (SCRAM).
    let supported =
        mechanisms.iter().any(|m| m == &requested) && mechanism_to_code(&requested).is_some();

    let error_code = if supported {
        if let Some(code) = mechanism_to_code(&requested) {
            cache.set_sasl_session(
                connection_id,
                SaslSession::AwaitingClientFirst { mechanism: code },
            );
        }
        0
    } else {
        ResponseError::UnsupportedSaslMechanism.code()
    };

    Some(KafkaPacket::SaslHandshakeResponse(
        SaslHandshakeResponse::default()
            .with_error_code(error_code)
            .with_mechanisms(mechanisms.into_iter().map(StrBytes::from).collect()),
    ))
}

pub fn process_sasl_authenticate(
    cache: &Arc<KafkaCacheManager>,
    connection_id: u64,
    req: &SaslAuthenticateRequest,
) -> Option<KafkaPacket> {
    let session = cache.get_sasl_session(connection_id);
    match session {
        Some(SaslSession::AwaitingClientFirst { mechanism }) => Some(handle_client_first(
            cache,
            connection_id,
            mechanism,
            &req.auth_bytes,
        )),
        Some(SaslSession::AwaitingClientFinal {
            mechanism,
            credential,
            client_first_bare,
            server_first,
        }) => Some(handle_client_final(
            cache,
            connection_id,
            mechanism,
            credential,
            client_first_bare,
            server_first,
            &req.auth_bytes,
        )),
        _ => Some(authenticate_error(
            ResponseError::IllegalSaslState.code(),
            "SaslAuthenticate received before a successful SaslHandshake",
        )),
    }
}

fn configured_mechanisms() -> Vec<String> {
    broker_config().kafka_runtime.sasl.mechanisms.clone()
}

fn authenticate_error(code: i16, message: &str) -> KafkaPacket {
    KafkaPacket::SaslAuthenticateResponse(
        SaslAuthenticateResponse::default()
            .with_error_code(code)
            .with_error_message(Some(StrBytes::from(message.to_string()))),
    )
}

fn authenticate_ok(auth_bytes: Bytes) -> KafkaPacket {
    KafkaPacket::SaslAuthenticateResponse(
        SaslAuthenticateResponse::default()
            .with_error_code(0)
            .with_auth_bytes(auth_bytes)
            // 0 = no server-imposed re-authentication window (KIP-368 deferred).
            .with_session_lifetime_ms(0),
    )
}

fn handle_client_first(
    cache: &Arc<KafkaCacheManager>,
    connection_id: u64,
    mechanism: i8,
    auth_bytes: &[u8],
) -> KafkaPacket {
    let parsed = match parse_client_first(auth_bytes) {
        Ok(p) => p,
        Err(e) => {
            cache.remove_sasl_session(connection_id);
            return authenticate_error(ResponseError::SaslAuthenticationFailed.code(), &e);
        }
    };

    let Some(credential) = cache.get_scram_credential(&parsed.username, mechanism) else {
        cache.remove_sasl_session(connection_id);
        // Same error for unknown user and bad password, so probing can't tell them apart.
        return authenticate_error(
            ResponseError::SaslAuthenticationFailed.code(),
            "authentication failed",
        );
    };

    let server_nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SERVER_NONCE_LEN)
        .map(char::from)
        .collect();
    let server_first = build_server_first(
        &parsed.client_nonce,
        &server_nonce,
        &credential.salt,
        credential.iterations,
    );

    cache.set_sasl_session(
        connection_id,
        SaslSession::AwaitingClientFinal {
            mechanism,
            credential,
            client_first_bare: parsed.bare,
            server_first: server_first.clone(),
        },
    );

    authenticate_ok(Bytes::from(server_first.into_bytes()))
}

#[allow(clippy::too_many_arguments)]
fn handle_client_final(
    cache: &Arc<KafkaCacheManager>,
    connection_id: u64,
    mechanism: i8,
    credential: KafkaScramCredential,
    client_first_bare: String,
    server_first: String,
    auth_bytes: &[u8],
) -> KafkaPacket {
    // The combined nonce the client must echo is embedded in server-first (r=...).
    let combined_nonce = server_first
        .strip_prefix("r=")
        .and_then(|rest| rest.split(',').next())
        .unwrap_or_default()
        .to_string();

    match verify_client_final(
        mechanism,
        &credential,
        &client_first_bare,
        &server_first,
        &combined_nonce,
        auth_bytes,
    ) {
        Ok(server_final) => {
            cache.set_sasl_session(
                connection_id,
                SaslSession::Authenticated {
                    principal: credential.user.clone(),
                },
            );
            authenticate_ok(Bytes::from(server_final.into_bytes()))
        }
        Err(e) => {
            warn!(
                "Kafka SASL authentication failed for connection {}: {}",
                connection_id, e
            );
            cache.remove_sasl_session(connection_id);
            authenticate_error(
                ResponseError::SaslAuthenticationFailed.code(),
                "authentication failed",
            )
        }
    }
}
