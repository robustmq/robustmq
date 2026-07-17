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
use protocol::meta::meta_service_amqp::{
    DeleteBindingReply, DeleteBindingRequest, DeleteExchangeReply, DeleteExchangeRequest,
    DeleteQueueReply, DeleteQueueRequest, ListBindingReply, ListBindingRequest, ListExchangeReply,
    ListExchangeRequest, ListQueueReply, ListQueueRequest, SetBindingReply, SetBindingRequest,
    SetExchangeReply, SetExchangeRequest, SetQueueReply, SetQueueRequest,
};

use crate::pool::ClientPool;

macro_rules! generate_amqp_service_call {
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

generate_amqp_service_call!(placement_set_exchange, SetExchangeRequest, SetExchangeReply);
generate_amqp_service_call!(
    placement_delete_exchange,
    DeleteExchangeRequest,
    DeleteExchangeReply
);
generate_amqp_service_call!(
    placement_list_exchange,
    ListExchangeRequest,
    ListExchangeReply
);
generate_amqp_service_call!(placement_set_queue, SetQueueRequest, SetQueueReply);
generate_amqp_service_call!(placement_delete_queue, DeleteQueueRequest, DeleteQueueReply);
generate_amqp_service_call!(placement_list_queue, ListQueueRequest, ListQueueReply);
generate_amqp_service_call!(placement_set_binding, SetBindingRequest, SetBindingReply);
generate_amqp_service_call!(
    placement_delete_binding,
    DeleteBindingRequest,
    DeleteBindingReply
);
generate_amqp_service_call!(placement_list_binding, ListBindingRequest, ListBindingReply);
