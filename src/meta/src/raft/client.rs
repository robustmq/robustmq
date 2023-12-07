/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use protocol::robust::meta::{
    meta_service_client::MetaServiceClient, FindLeaderReply, FindLeaderRequest,
};

pub async fn find_leader(addr: &String) -> FindLeaderReply {
    let mut client = MetaServiceClient::connect(format!("http://{}", addr))
        .await
        .unwrap();
    let request = tonic::Request::new(FindLeaderRequest {});
    let response = client.find_leader(request).await.unwrap();
    return response.into_inner();
}

pub fn vote() {}

pub fn transform_leader() {}

pub fn heartbeat() {}
