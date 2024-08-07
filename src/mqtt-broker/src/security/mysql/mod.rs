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


use super::AuthStorageAdapter;
use axum::async_trait;
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use metadata_struct::mqtt::user::MQTTUser;
use mysql::Pool;
use third_driver::mysql::build_mysql_conn_pool;

mod schema;
pub struct MySQLAuthStorageAdapter {
    conn: Pool,
}

impl MySQLAuthStorageAdapter {
    pub fn new(addr: String) -> Self {
        let poll = match build_mysql_conn_pool(&addr) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        return MySQLAuthStorageAdapter { conn: poll };
    }
}

#[async_trait]
impl AuthStorageAdapter for MySQLAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MQTTUser>, RobustMQError> {
        let sql = "select * from mqtt_user";
        return Ok(DashMap::with_capacity(2));
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        return Ok(None);
    }
}
