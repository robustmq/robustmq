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
    create_path, delete_path, get_path, list_path, update_path, server::HttpServerState, v1_path,
};
use axum::routing::{delete, get, post, put};
use axum::Router;
use cluster::{get_cluster, list_cluster, set_cluster};
use user::{create_user, delete_user, get_user, list_user, update_user};

pub mod cluster;
pub mod user;

pub const ROUTE_MQTT_USER: &str = "/mqtt/user";
pub const ROUTE_MQTT_ACL: &str = "/mqtt/acl";
pub const ROUTE_MQTT_SESSION: &str = "/mqtt/session";
pub const ROUTE_CLUSTER: &str = "/mqtt/cluster";

pub fn mqtt_routes() -> Router<HttpServerState> {
    return Router::new()
        // user
        .route(&v1_path(&get_path(ROUTE_MQTT_USER)), get(get_user))
        .route(&v1_path(&create_path(ROUTE_MQTT_USER)), post(create_user))
        .route(&v1_path(&update_path(ROUTE_MQTT_USER)), put(update_user))
        .route(&v1_path(&delete_path(ROUTE_MQTT_USER)), delete(delete_user))
        .route(&v1_path(&list_path(ROUTE_MQTT_USER)), get(list_user))
        
        // acl
        .route(&v1_path(&get_path(ROUTE_MQTT_ACL)), get(get_cluster))
        .route(&v1_path(&update_path(ROUTE_MQTT_ACL)), put(set_cluster))
        .route(&v1_path(&list_path(ROUTE_MQTT_ACL)), get(list_cluster))
        
        // session
        .route(&v1_path(&get_path(ROUTE_MQTT_SESSION)), get(get_cluster))
        .route(&v1_path(&update_path(ROUTE_MQTT_SESSION)), put(set_cluster))
        .route(&v1_path(&list_path(ROUTE_MQTT_SESSION)), get(list_cluster))
        
        // cluster
        .route(&v1_path(&get_path(ROUTE_CLUSTER)), get(get_cluster))
        .route(&v1_path(&update_path(ROUTE_CLUSTER)), put(set_cluster))
        .route(&v1_path(&list_path(ROUTE_CLUSTER)), get(list_cluster));
}
