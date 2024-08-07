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
        return Ok(DashMap::with_capacity(2));
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        return Ok(None);
    }
}
