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

use protocol::meta::meta_service_kafka::kafka_service_client::KafkaServiceClient;
use protocol::meta::meta_service_kafka::{
    DeleteKafkaDelegationTokenReply, DeleteKafkaDelegationTokenRequest, DeleteKafkaQuotaReply,
    DeleteKafkaQuotaRequest, DeleteScramCredentialReply, DeleteScramCredentialRequest,
    GetCoordinatorLeaderReply, GetCoordinatorLeaderRequest, ListKafkaDelegationTokenReply,
    ListKafkaDelegationTokenRequest, ListKafkaQuotaReply, ListKafkaQuotaRequest,
    ListScramCredentialReply, ListScramCredentialRequest, SetKafkaDelegationTokenReply,
    SetKafkaDelegationTokenRequest, SetKafkaQuotaReply, SetKafkaQuotaRequest,
    SetScramCredentialReply, SetScramCredentialRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    GetCoordinatorLeaderRequest,
    KafkaServiceClient<Channel>,
    GetCoordinatorLeaderReply,
    get_coordinator_leader,
    "KafkaService",
    "GetCoordinatorLeader",
    true
);

impl_retriable_request!(
    SetKafkaQuotaRequest,
    KafkaServiceClient<Channel>,
    SetKafkaQuotaReply,
    set_kafka_quota,
    "KafkaService",
    "SetKafkaQuota",
    true
);

impl_retriable_request!(
    DeleteKafkaQuotaRequest,
    KafkaServiceClient<Channel>,
    DeleteKafkaQuotaReply,
    delete_kafka_quota,
    "KafkaService",
    "DeleteKafkaQuota",
    true
);

impl_retriable_request!(
    ListKafkaQuotaRequest,
    KafkaServiceClient<Channel>,
    ListKafkaQuotaReply,
    list_kafka_quota,
    "KafkaService",
    "ListKafkaQuota",
    true
);

impl_retriable_request!(
    SetKafkaDelegationTokenRequest,
    KafkaServiceClient<Channel>,
    SetKafkaDelegationTokenReply,
    set_kafka_delegation_token,
    "KafkaService",
    "SetKafkaDelegationToken",
    true
);

impl_retriable_request!(
    DeleteKafkaDelegationTokenRequest,
    KafkaServiceClient<Channel>,
    DeleteKafkaDelegationTokenReply,
    delete_kafka_delegation_token,
    "KafkaService",
    "DeleteKafkaDelegationToken",
    true
);

impl_retriable_request!(
    ListKafkaDelegationTokenRequest,
    KafkaServiceClient<Channel>,
    ListKafkaDelegationTokenReply,
    list_kafka_delegation_token,
    "KafkaService",
    "ListKafkaDelegationToken",
    true
);

impl_retriable_request!(
    SetScramCredentialRequest,
    KafkaServiceClient<Channel>,
    SetScramCredentialReply,
    set_scram_credential,
    "KafkaService",
    "SetScramCredential",
    true
);

impl_retriable_request!(
    DeleteScramCredentialRequest,
    KafkaServiceClient<Channel>,
    DeleteScramCredentialReply,
    delete_scram_credential,
    "KafkaService",
    "DeleteScramCredential",
    true
);

impl_retriable_request!(
    ListScramCredentialRequest,
    KafkaServiceClient<Channel>,
    ListScramCredentialReply,
    list_scram_credential,
    "KafkaService",
    "ListScramCredential",
    true
);
