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
use r2d2_mysql::{
    mysql::{Opts, OptsBuilder},
    r2d2::Pool,
    MySqlConnectionManager,
};

pub type MysqlPool = Pool<MySqlConnectionManager>;

pub fn build_mysql_conn_pool(addr: &str) -> Result<MysqlPool, CommonError> {
    let opts = Opts::from_url(addr)
        .map_err(|e| CommonError::CommonError(format!("Invalid MySQL connection string: {}", e)))?;

    let builder = OptsBuilder::from_opts(opts);
    let manager = MySqlConnectionManager::new(builder);

    match Pool::new(manager) {
        Ok(pool) => Ok(pool),
        Err(e) => Err(CommonError::CommonError(e.to_string())),
    }
}
