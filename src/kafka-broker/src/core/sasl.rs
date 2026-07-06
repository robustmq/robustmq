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

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hmac::{Hmac, Mac};
use metadata_struct::kafka::scram::{
    KafkaScramCredential, SCRAM_MECHANISM_SHA_256, SCRAM_MECHANISM_SHA_512,
};
use sha2::{Digest, Sha256, Sha512};

pub const MECHANISM_SCRAM_SHA_256: &str = "SCRAM-SHA-256";
pub const MECHANISM_SCRAM_SHA_512: &str = "SCRAM-SHA-512";

pub fn mechanism_to_code(mechanism: &str) -> Option<i8> {
    match mechanism {
        MECHANISM_SCRAM_SHA_256 => Some(SCRAM_MECHANISM_SHA_256),
        MECHANISM_SCRAM_SHA_512 => Some(SCRAM_MECHANISM_SHA_512),
        _ => None,
    }
}

pub fn code_to_mechanism(code: i8) -> Option<&'static str> {
    match code {
        SCRAM_MECHANISM_SHA_256 => Some(MECHANISM_SCRAM_SHA_256),
        SCRAM_MECHANISM_SHA_512 => Some(MECHANISM_SCRAM_SHA_512),
        _ => None,
    }
}

// Per-connection SASL progress. Present only while SASL is enabled; a connection
// with no entry (or Authenticated) is the two terminal states the request gate
// cares about.
#[derive(Clone)]
pub enum SaslSession {
    // Handshake done, mechanism chosen, awaiting the SCRAM client-first message.
    AwaitingClientFirst {
        mechanism: i8,
    },
    // server-first sent; everything needed to verify the client-final message.
    AwaitingClientFinal {
        mechanism: i8,
        credential: KafkaScramCredential,
        client_first_bare: String,
        server_first: String,
    },
    Authenticated {
        principal: String,
    },
}

pub struct ParsedClientFirst {
    pub username: String,
    pub client_nonce: String,
    pub bare: String,
}

// client-first-message = gs2-header + client-first-message-bare. We only support
// the "no channel binding, no authzid" header ("n,,"); the bare is n=user,r=nonce.
pub fn parse_client_first(bytes: &[u8]) -> Result<ParsedClientFirst, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "client-first is not UTF-8".to_string())?;

    // Skip the gs2 header: two comma-separated fields before the bare.
    let mut comma_iter = text.match_indices(',');
    comma_iter.next();
    let second = comma_iter
        .next()
        .ok_or_else(|| "malformed gs2 header".to_string())?
        .0;
    let bare = &text[second + 1..];

    let mut username = None;
    let mut client_nonce = None;
    for field in bare.split(',') {
        if let Some(v) = field.strip_prefix("n=") {
            username = Some(scram_unescape(v));
        } else if let Some(v) = field.strip_prefix("r=") {
            client_nonce = Some(v.to_string());
        }
    }

    Ok(ParsedClientFirst {
        username: username.ok_or_else(|| "missing username in client-first".to_string())?,
        client_nonce: client_nonce.ok_or_else(|| "missing nonce in client-first".to_string())?,
        bare: bare.to_string(),
    })
}

// SCRAM username escaping: =2C -> ',', =3D -> '='.
fn scram_unescape(value: &str) -> String {
    value.replace("=2C", ",").replace("=3D", "=")
}

pub fn build_server_first(
    client_nonce: &str,
    server_nonce: &str,
    salt: &[u8],
    iterations: i32,
) -> String {
    format!(
        "r={}{},s={},i={}",
        client_nonce,
        server_nonce,
        BASE64.encode(salt),
        iterations
    )
}

// Verify the client-final message against the stored credential and return the
// server-final message (v=<ServerSignature>) on success.
pub fn verify_client_final(
    session_mechanism: i8,
    credential: &KafkaScramCredential,
    client_first_bare: &str,
    server_first: &str,
    combined_nonce_expected: &str,
    client_final: &[u8],
) -> Result<String, String> {
    let text =
        std::str::from_utf8(client_final).map_err(|_| "client-final is not UTF-8".to_string())?;

    let mut channel_binding = None;
    let mut nonce = None;
    let mut proof_b64 = None;
    for field in text.split(',') {
        if let Some(v) = field.strip_prefix("c=") {
            channel_binding = Some(v);
        } else if let Some(v) = field.strip_prefix("r=") {
            nonce = Some(v);
        } else if let Some(v) = field.strip_prefix("p=") {
            proof_b64 = Some(v);
        }
    }

    // "biws" is base64("n,,") — the only channel binding we accept.
    if channel_binding != Some("biws") {
        return Err("unsupported channel binding".to_string());
    }
    if nonce != Some(combined_nonce_expected) {
        return Err("nonce mismatch".to_string());
    }
    let proof = BASE64
        .decode(proof_b64.ok_or_else(|| "missing client proof".to_string())?)
        .map_err(|_| "client proof is not valid base64".to_string())?;

    let client_final_without_proof = text
        .rsplit_once(",p=")
        .map(|(head, _)| head)
        .ok_or_else(|| "malformed client-final".to_string())?;
    let auth_message = format!(
        "{},{},{}",
        client_first_bare, server_first, client_final_without_proof
    );

    let (client_signature, server_signature, proof_len_ok) = match session_mechanism {
        SCRAM_MECHANISM_SHA_256 => (
            hmac_sha256(&credential.stored_key, auth_message.as_bytes()),
            hmac_sha256(&credential.server_key, auth_message.as_bytes()),
            proof.len() == 32,
        ),
        SCRAM_MECHANISM_SHA_512 => (
            hmac_sha512(&credential.stored_key, auth_message.as_bytes()),
            hmac_sha512(&credential.server_key, auth_message.as_bytes()),
            proof.len() == 64,
        ),
        _ => return Err("unsupported mechanism".to_string()),
    };
    if !proof_len_ok {
        return Err("client proof has wrong length".to_string());
    }

    // ClientKey = ClientProof XOR ClientSignature; the password is correct iff
    // H(ClientKey) == StoredKey.
    let client_key: Vec<u8> = proof
        .iter()
        .zip(client_signature.iter())
        .map(|(a, b)| a ^ b)
        .collect();
    let derived_stored = match session_mechanism {
        SCRAM_MECHANISM_SHA_256 => Sha256::digest(&client_key).to_vec(),
        _ => Sha512::digest(&client_key).to_vec(),
    };
    if derived_stored != credential.stored_key {
        return Err("authentication failed".to_string());
    }

    Ok(format!("v={}", BASE64.encode(server_signature)))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha512>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reproduce what the client does so the full round trip can be tested:
    // SaltedPassword = PBKDF2(password, salt, iters); ClientKey = HMAC(SP, "Client Key");
    // StoredKey = H(ClientKey); ServerKey = HMAC(SP, "Server Key").
    fn salted_password_sha256(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
        pbkdf2::pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, iterations).to_vec()
    }

    fn credential_for(password: &str, salt: &[u8], iterations: i32) -> KafkaScramCredential {
        let sp = salted_password_sha256(password, salt, iterations as u32);
        let client_key = hmac_sha256(&sp, b"Client Key");
        let stored_key = Sha256::digest(&client_key).to_vec();
        let server_key = hmac_sha256(&sp, b"Server Key");
        KafkaScramCredential {
            tenant: "default".to_string(),
            user: "alice".to_string(),
            mechanism: SCRAM_MECHANISM_SHA_256,
            iterations,
            salt: salt.to_vec(),
            stored_key,
            server_key,
        }
    }

    fn client_proof(password: &str, salt: &[u8], iterations: u32, auth_message: &str) -> String {
        let sp = salted_password_sha256(password, salt, iterations);
        let client_key = hmac_sha256(&sp, b"Client Key");
        let stored_key = Sha256::digest(&client_key);
        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let proof: Vec<u8> = client_key
            .iter()
            .zip(client_signature.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        BASE64.encode(proof)
    }

    #[test]
    fn parse_client_first_extracts_user_and_nonce() {
        let parsed = parse_client_first(b"n,,n=alice,r=clientNONCE").unwrap();
        assert_eq!(parsed.username, "alice");
        assert_eq!(parsed.client_nonce, "clientNONCE");
        assert_eq!(parsed.bare, "n=alice,r=clientNONCE");
    }

    #[test]
    fn full_scram_sha256_round_trip_succeeds() {
        let salt = b"saltsalt";
        let iterations = 4096;
        let credential = credential_for("pencil", salt, iterations);

        let client_nonce = "rOprNGfwEbeRWgbNEkqO";
        let server_nonce = "SERVERnonce123";
        let client_first_bare = format!("n=alice,r={}", client_nonce);
        let server_first = build_server_first(client_nonce, server_nonce, salt, iterations);
        let combined = format!("{}{}", client_nonce, server_nonce);

        let client_final_without_proof = format!("c=biws,r={}", combined);
        let auth_message = format!(
            "{},{},{}",
            client_first_bare, server_first, client_final_without_proof
        );
        let proof = client_proof("pencil", salt, iterations as u32, &auth_message);
        let client_final = format!("{},p={}", client_final_without_proof, proof);

        let server_final = verify_client_final(
            SCRAM_MECHANISM_SHA_256,
            &credential,
            &client_first_bare,
            &server_first,
            &combined,
            client_final.as_bytes(),
        )
        .unwrap();
        assert!(server_final.starts_with("v="));
    }

    #[test]
    fn wrong_password_is_rejected() {
        let salt = b"saltsalt";
        let iterations = 4096;
        let credential = credential_for("pencil", salt, iterations);

        let client_nonce = "abc";
        let server_nonce = "def";
        let client_first_bare = format!("n=alice,r={}", client_nonce);
        let server_first = build_server_first(client_nonce, server_nonce, salt, iterations);
        let combined = format!("{}{}", client_nonce, server_nonce);
        let client_final_without_proof = format!("c=biws,r={}", combined);
        let auth_message = format!(
            "{},{},{}",
            client_first_bare, server_first, client_final_without_proof
        );
        let proof = client_proof("wrong", salt, iterations as u32, &auth_message);
        let client_final = format!("{},p={}", client_final_without_proof, proof);

        let result = verify_client_final(
            SCRAM_MECHANISM_SHA_256,
            &credential,
            &client_first_bare,
            &server_first,
            &combined,
            client_final.as_bytes(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn nonce_tampering_is_rejected() {
        let salt = b"saltsalt";
        let credential = credential_for("pencil", salt, 4096);
        let client_final = "c=biws,r=TAMPERED,p=AAAA";
        let result = verify_client_final(
            SCRAM_MECHANISM_SHA_256,
            &credential,
            "n=alice,r=abc",
            "r=abcdef,s=c2FsdA==,i=4096",
            "abcdef",
            client_final.as_bytes(),
        );
        assert!(result.is_err());
    }
}
