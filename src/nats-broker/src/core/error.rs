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

use common_base::error::common::CommonError;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum NatsBrokerError {
    #[error("{0}")]
    FromCommonError(Box<CommonError>),

    #[error("{0}")]
    CommonError(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(u64),

    #[error("Queue group empty, stopping task: {0}")]
    QueueGroupEmpty(String),

    #[error("{0}")]
    FromSerdeJsonError(#[from] serde_json::Error),
}

impl From<CommonError> for NatsBrokerError {
    fn from(e: CommonError) -> Self {
        NatsBrokerError::FromCommonError(Box::new(e))
    }
}

impl From<NatsBrokerError> for Status {
    fn from(e: NatsBrokerError) -> Self {
        Status::cancelled(e.to_string())
    }
}

/// Protocol-level errors sent to the client as `-ERR '<message>'`.
///
/// Variants are grouped by whether the server closes the connection after
/// sending the error (`disconnect = true`) or keeps it open.
#[derive(Debug, Clone, PartialEq)]
pub enum NatsProtocolError {
    // ── Disconnect errors ────────────────────────────────────────────────
    UnknownProtocolOperation,
    AttemptedToConnectToRoutePort,
    AuthorizationViolation,
    AuthorizationTimeout,
    InvalidClientProtocol,
    MaximumControlLineExceeded,
    ParserError,
    SecureConnectionTlsRequired,
    StaleConnection,
    MaximumConnectionsExceeded,
    SlowConsumer,
    MaximumPayloadViolation,

    // ── Non-disconnect errors ─────────────────────────────────────────────
    InvalidSubject,
    PermissionsViolationForSubscription(String),
    PermissionsViolationForPublish(String),
}

impl NatsProtocolError {
    /// Returns `true` if the server must close the connection after sending
    /// this error.
    pub fn should_disconnect(&self) -> bool {
        matches!(
            self,
            NatsProtocolError::UnknownProtocolOperation
                | NatsProtocolError::AttemptedToConnectToRoutePort
                | NatsProtocolError::AuthorizationViolation
                | NatsProtocolError::AuthorizationTimeout
                | NatsProtocolError::InvalidClientProtocol
                | NatsProtocolError::MaximumControlLineExceeded
                | NatsProtocolError::ParserError
                | NatsProtocolError::SecureConnectionTlsRequired
                | NatsProtocolError::StaleConnection
                | NatsProtocolError::MaximumConnectionsExceeded
                | NatsProtocolError::SlowConsumer
                | NatsProtocolError::MaximumPayloadViolation
        )
    }

    /// Returns the error message string sent in `-ERR '<message>'`.
    pub fn message(&self) -> String {
        match self {
            NatsProtocolError::UnknownProtocolOperation => "Unknown Protocol Operation".to_string(),
            NatsProtocolError::AttemptedToConnectToRoutePort => {
                "Attempted To Connect To Route Port".to_string()
            }
            NatsProtocolError::AuthorizationViolation => "Authorization Violation".to_string(),
            NatsProtocolError::AuthorizationTimeout => "Authorization Timeout".to_string(),
            NatsProtocolError::InvalidClientProtocol => "Invalid Client Protocol".to_string(),
            NatsProtocolError::MaximumControlLineExceeded => {
                "Maximum Control Line Exceeded".to_string()
            }
            NatsProtocolError::ParserError => "Parser Error".to_string(),
            NatsProtocolError::SecureConnectionTlsRequired => {
                "Secure Connection - TLS Required".to_string()
            }
            NatsProtocolError::StaleConnection => "Stale Connection".to_string(),
            NatsProtocolError::MaximumConnectionsExceeded => {
                "Maximum Connections Exceeded".to_string()
            }
            NatsProtocolError::SlowConsumer => "Slow Consumer".to_string(),
            NatsProtocolError::MaximumPayloadViolation => "Maximum Payload Violation".to_string(),
            NatsProtocolError::InvalidSubject => "Invalid Subject".to_string(),
            NatsProtocolError::PermissionsViolationForSubscription(subject) => {
                format!("Permissions Violation for Subscription to {}", subject)
            }
            NatsProtocolError::PermissionsViolationForPublish(subject) => {
                format!("Permissions Violation for Publish to {}", subject)
            }
        }
    }
}
