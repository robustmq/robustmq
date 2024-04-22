use crate::{
    record::Record,
    storage::{ShardConfig, StorageAdapter},
};
use axum::async_trait;
use common_base::{errors::RobustMQError, tools::now_second};
use mysql::{params, prelude::Queryable, Pool};

use self::schema::TMqttKvMsg;
pub mod schema;

#[derive(Clone)]
pub struct MySQLStorageAdapter {
    pool: Pool,
}

impl MySQLStorageAdapter {
    pub fn new(pool: Pool) -> Self {
        return MySQLStorageAdapter { pool };
    }

    pub fn shard_table_name(&self, shard_name: String) -> String {
        return format!("t_mqtt_shard_{}", shard_name);
    }
}

#[async_trait]
impl StorageAdapter for MySQLStorageAdapter {
    async fn create_shard(
        &self,
        shard_name: String,
        shard_config: ShardConfig,
    ) -> Result<(), RobustMQError> {
        let table = self.shard_table_name(shard_name);
        // Check whether the table exists

        // Create a table if it does not exist
        return return Ok(());
    }

    async fn delete_shard(&self, shard_name: String) -> Result<(), RobustMQError> {
        // Check whether the table exists

        // Delete a table if it exists
        return Ok(());
    }
    async fn set(&self, key: String, value: Record) -> Result<(), RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let values = vec![TMqttKvMsg {
                    key,
                    value: String::from_utf8(value.data).unwrap(),
                    create_time: now_second(),
                }];
                match conn.exec_batch(
                    r"INSERT INTO storage_kv(data_key,data_value,create_time) VALUES (:key,:value,:create_time)",
                    values.iter().map(|p| {
                        params! {
                            "key" => p.key.clone(),
                            "value" => p.value.clone(),
                            "create_time" => p.create_time,
                        }
                    }),
                ) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(RobustMQError::CommmonError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    async fn get(&self, key: String) -> Result<Option<Record>, RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!("select data_value from storage_kv where data_key='{}'", key);
                match conn.query(sql) {
                    Ok(data) => {
                        if data.len() > 0{
                            return Ok(Some(Record::build_e(data.get(0).unwrap().try_into().unwrap())));
                        }
                        return Ok(None);
                    }
                    Err(e) => {
                        return Err(RobustMQError::CommmonError(e.to_string()));
                    }
                }
                // let res: Vec<(String)> = .unwrap();
                return Ok(None);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }
    async fn delete(&self, key: String) -> Result<(), RobustMQError> {
        return Ok(());
    }
    async fn exists(&self, key: String) -> Result<bool, RobustMQError> {
        return Ok(false);
        // match self.pool.get_conn() {
        //     Ok(mut conn) => {
        //         match conn.query_first(
        //             r"select count(*) as count from t_mqtt_config where key=?",
        //             key,
        //         ) {
        //             Ok(_) => {
        //                 return Ok(());
        //             }
        //             Err(e) => {
        //                 return Err(RobustMQError::CommmonError(e.to_string()));
        //             }
        //         }
        //     }
        //     Err(e) => {
        //         return Err(RobustMQError::CommmonError(e.to_string()));
        //     }
        // }
    }

    async fn stream_write(&self, _: String, _: Vec<Record>) -> Result<Vec<usize>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read(
        &self,
        _: String,
        _: String,
        _: Option<u128>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_commit_offset(
        &self,
        _: String,
        _: String,
        _: u128,
    ) -> Result<bool, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_offset(
        &self,
        _: String,
        _: usize,
    ) -> Result<Option<Record>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_timestamp(
        &self,
        _: String,
        _: u128,
        _: u128,
        _: Option<usize>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_key(
        &self,
        _: String,
        _: String,
    ) -> Result<Option<Record>, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }
}

fn build_mysql_conn_pool(addr: &str) -> Result<Pool, RobustMQError> {
    match Pool::new(addr) {
        Ok(pool) => return Ok(pool),
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{record::Record, storage::StorageAdapter};

    use super::{build_mysql_conn_pool, MySQLStorageAdapter};

    #[tokio::test]
    async fn mysql_set() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let key = String::from("name");
        let value = String::from("loboxu");
        mysql_adapter
            .set(key, Record::build_e(value))
            .await
            .unwrap();
    }
}
