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

use bytes::Bytes;
use protocol::nats::packet::NatsPacket;
use tracing::info;

/// Handle a PUB packet from the client.
///
/// Responsibilities (all TODO):
/// - Validate subject name
/// - Look up all active subscriptions matching the subject (including wildcard)
/// - Fan-out: deliver MSG to each matching subscriber connection
/// - If reply_to is set, include it in the delivered MSG
pub fn process_pub(
    subject: &str,
    reply_to: Option<&str>,
    payload: &Bytes,
) -> Option<NatsPacket> {
    info!(
        subject,
        reply_to,
        payload = %String::from_utf8_lossy(payload),
        "PUB received"
    );
    None
}
