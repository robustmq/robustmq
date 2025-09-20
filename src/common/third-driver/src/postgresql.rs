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
use r2d2_postgres::{
    postgres::{Config, NoTls},
    r2d2::Pool,
    PostgresConnectionManager,
};

pub type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;

pub fn build_postgresql_conn_pool(addr: &str) -> Result<PostgresPool, CommonError> {
    let config = addr.parse::<Config>().map_err(|e| {
        CommonError::CommonError(format!("Invalid PostgreSQL connection string: {}", e))
    })?;

    let manager = PostgresConnectionManager::new(config, NoTls);

    match Pool::new(manager) {
        Ok(pool) => Ok(pool),
        Err(e) => Err(CommonError::CommonError(e.to_string())),
    }
}
