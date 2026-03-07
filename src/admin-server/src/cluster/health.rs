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

use common_base::http_response::success_response;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct HealthCheckResp {
    status: String,
    check_type: String,
    message: String,
}

fn build_placeholder_resp(check_type: &str) -> String {
    success_response(HealthCheckResp {
        status: "ok".to_string(),
        check_type: check_type.to_string(),
        message: "health check placeholder".to_string(),
    })
}

pub async fn health_live() -> String {
    build_placeholder_resp("live")
}

pub async fn health_ready() -> String {
    build_placeholder_resp("ready")
}

pub async fn health_startup() -> String {
    build_placeholder_resp("startup")
}

pub async fn health_node() -> String {
    build_placeholder_resp("node")
}

pub async fn health_cluster() -> String {
    build_placeholder_resp("cluster")
}
