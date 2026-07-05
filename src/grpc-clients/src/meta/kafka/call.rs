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
use protocol::meta::meta_service_kafka::{
    DeleteKafkaDelegationTokenReply, DeleteKafkaDelegationTokenRequest, DeleteKafkaQuotaReply,
    DeleteKafkaQuotaRequest, DeleteScramCredentialReply, DeleteScramCredentialRequest,
    GetCoordinatorLeaderReply, GetCoordinatorLeaderRequest, ListKafkaDelegationTokenReply,
    ListKafkaDelegationTokenRequest, ListKafkaQuotaReply, ListKafkaQuotaRequest,
    ListScramCredentialReply, ListScramCredentialRequest, SetKafkaDelegationTokenReply,
    SetKafkaDelegationTokenRequest, SetKafkaQuotaReply, SetKafkaQuotaRequest,
    SetScramCredentialReply, SetScramCredentialRequest,
};

use crate::pool::ClientPool;

macro_rules! generate_kafka_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty) => {
        pub async fn $fn_name(
            client_pool: &ClientPool,
            addrs: &[impl AsRef<str>],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            $crate::utils::retry_call(client_pool, addrs, request).await
        }
    };
}

generate_kafka_service_call!(
    get_coordinator_leader,
    GetCoordinatorLeaderRequest,
    GetCoordinatorLeaderReply
);
generate_kafka_service_call!(set_kafka_quota, SetKafkaQuotaRequest, SetKafkaQuotaReply);
generate_kafka_service_call!(
    delete_kafka_quota,
    DeleteKafkaQuotaRequest,
    DeleteKafkaQuotaReply
);
generate_kafka_service_call!(list_kafka_quota, ListKafkaQuotaRequest, ListKafkaQuotaReply);
generate_kafka_service_call!(
    set_kafka_delegation_token,
    SetKafkaDelegationTokenRequest,
    SetKafkaDelegationTokenReply
);
generate_kafka_service_call!(
    delete_kafka_delegation_token,
    DeleteKafkaDelegationTokenRequest,
    DeleteKafkaDelegationTokenReply
);
generate_kafka_service_call!(
    list_kafka_delegation_token,
    ListKafkaDelegationTokenRequest,
    ListKafkaDelegationTokenReply
);
generate_kafka_service_call!(
    set_scram_credential,
    SetScramCredentialRequest,
    SetScramCredentialReply
);
generate_kafka_service_call!(
    delete_scram_credential,
    DeleteScramCredentialRequest,
    DeleteScramCredentialReply
);
generate_kafka_service_call!(
    list_scram_credential,
    ListScramCredentialRequest,
    ListScramCredentialReply
);
