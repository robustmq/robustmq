// Copyright 2023 RobustMQ Team
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


use super::{
    path_create, path_delete, path_get, path_list, path_update, server::HttpServerState, v1_path,
};
use axum::routing::{delete, get, post, put};
use axum::Router;
use cluster::{cluster_get, cluster_list, cluster_set};
use user::{user_create, user_delete, user_get, user_list, user_update};

pub mod cluster;
pub mod user;

pub const ROUTE_MQTT_USER: &str = "/mqtt/user";
pub const ROUTE_MQTT_ACL: &str = "/mqtt/acl";
pub const ROUTE_MQTT_SESSION: &str = "/mqtt/session";
pub const ROUTE_CLUSTER: &str = "/mqtt/cluster";

pub fn mqtt_routes() -> Router<HttpServerState> {
    return Router::new()
        // user
        .route(&v1_path(&path_get(ROUTE_MQTT_USER)), get(user_get))
        .route(&v1_path(&path_create(ROUTE_MQTT_USER)), post(user_create))
        .route(&v1_path(&path_update(ROUTE_MQTT_USER)), put(user_update))
        .route(&v1_path(&path_delete(ROUTE_MQTT_USER)), delete(user_delete))
        .route(&v1_path(&path_list(ROUTE_MQTT_USER)), get(user_list))
        
        // acl

        // session
        
        // cluster
        .route(&v1_path(&path_get(ROUTE_CLUSTER)), get(cluster_get))
        .route(&v1_path(&path_update(ROUTE_CLUSTER)), put(cluster_set))
        .route(&v1_path(&path_list(ROUTE_CLUSTER)), get(cluster_list));
}
