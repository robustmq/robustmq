use crate::{record::Record, storage::StorageAdapter};
use axum::async_trait;
use common_base::{errors::RobustMQError, tools::now_mills};
use mysql::{params, prelude::Queryable, Pool, PooledConn};

use self::schema::TMqttKvMsg;
pub mod schema;

// addr: mysql://root:password@localhost:3307/db_name
#[derive(Clone)]
pub struct MySQLStorageAdapter {
    pool: Pool,
}

impl MySQLStorageAdapter {
    pub fn new(pool: Pool) -> Self {
        return MySQLStorageAdapter { pool };
    }

    pub fn insert(&self) {
        // conn.ex
    }

    pub fn drop(&self) {}

    pub fn update(&self) {}

    pub fn select(&self) {}
}

#[async_trait]
impl StorageAdapter for MySQLStorageAdapter {
    async fn set(&self, key: String, value: Record) -> Result<(), RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let values = vec![TMqttKvMsg {
                    key,
                    value: String::from_utf8(value.data).unwrap(),
                    create_time: now_mills(),
                }];
                match conn.exec_batch(
                    r"INSERT INTO payment (key, value, create_time)
                      VALUES (:key, :value, :create_time)",
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
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }
    async fn delete(&self, key: String) -> Result<(), RobustMQError> {
        return Ok(());
    }
    async fn exists(&self, key: String) -> Result<bool, RobustMQError> {
        return Err(RobustMQError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
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

fn build_mysql_conn_pool(addr: &str) -> Result<PooledConn, RobustMQError> {
    match Pool::new(addr) {
        Ok(pool) => match pool.get_conn() {
            Ok(conn) => {
                return Ok(conn);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        },
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {}
