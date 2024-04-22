use crate::{
    record::Record,
    storage::{ShardConfig, StorageAdapter},
};
use axum::async_trait;
use common_base::{errors::RobustMQError, tools::now_second};
use mysql::{params, prelude::Queryable, Pool};

use self::schema::{TMqttKvMsg, TMqttRecord};
pub mod schema;

#[derive(Clone)]
pub struct MySQLStorageAdapter {
    pool: Pool,
}

impl MySQLStorageAdapter {
    pub fn new(pool: Pool) -> Self {
        return MySQLStorageAdapter { pool };
    }

    pub fn storage_record_table(&self, shard_name: String) -> String {
        return format!("storage_record_{}", shard_name);
    }

    pub fn storage_kv_table(&self) -> String {
        return format!("storage_kv");
    }
}

#[async_trait]
impl StorageAdapter for MySQLStorageAdapter {
    async fn create_shard(
        &self,
        shard_name: String,
        shard_config: ShardConfig,
    ) -> Result<(), RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let show_table_sql = format!(
                    "CREATE TABLE IF NOT EXISTS `{}` (
                    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
                    `msgid` varchar(64) DEFAULT NULL,
                    `payload` blob,
                    `create_time` int(11) NOT NULL,
                    PRIMARY KEY (`id`)
                    ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
                    self.storage_record_table(shard_name)
                );
                match conn.query_drop(show_table_sql) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    async fn delete_shard(&self, shard_name: String) -> Result<(), RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let show_table_sql = format!(
                    "DROP TABLE IF EXISTS `{}`",
                    self.storage_record_table(shard_name)
                );
                match conn.query_drop(show_table_sql) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    async fn set(&self, key: String, value: Record) -> Result<(), RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let values = vec![TMqttKvMsg {
                    key,
                    value: value.data,
                    create_time: now_second(),
                }];
                match conn.exec_batch(
                    format!("INSERT INTO {}(data_key,data_value,create_time) VALUES (:key,:value,:create_time)",self.storage_kv_table()),
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
                let sql = format!(
                    "select data_value from {} where data_key='{}'",
                    self.storage_kv_table(),
                    key
                );
                let data: Vec<String> = conn.query(sql).unwrap();
                if let Some(value) = data.first() {
                    return Ok(Some(Record::build_e(value.clone())));
                }
                return Ok(None);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    async fn delete(&self, key: String) -> Result<(), RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "delete from {} where data_key='{}'",
                    self.storage_kv_table(),
                    key
                );
                match conn.query_drop(sql) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    async fn exists(&self, key: String) -> Result<bool, RobustMQError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "select count(*) as count from {} where data_key='{}'",
                    self.storage_kv_table(),
                    key
                );
                let data: Vec<u32> = conn.query(sql).unwrap();
                if let Some(value) = data.first() {
                    return Ok(value.clone() > 0);
                }
                return Ok(false);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    async fn stream_write(
        &self,
        shard_name: String,
        data: Vec<Record>,
    ) -> Result<Vec<usize>, RobustMQError> {
        if data.len() == 0 {
            return Ok(Vec::new());
        }
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let mut values = Vec::new();
                for raw in data {
                    match serde_json::to_vec(&raw) {
                        Ok(s) => {
                            values.push(TMqttRecord {
                                msgid: String::from(""),
                                payload: s,
                                create_time: now_second(),
                            });
                        }
                        Err(e) => {
                            return Err(RobustMQError::CommmonError(e.to_string()));
                        }
                    }
                }

                match conn.exec_batch(
                    format!("INSERT INTO {}(msgid,payload,create_time) VALUES (:msgid,:payload,:create_time)",self.storage_record_table(shard_name)),
                    values.iter().map(|p| {
                        params! {
                            "msgid" => p.msgid.clone(),
                            "payload" => p.payload.clone(),
                            "create_time" => p.create_time,
                        }
                    }),
                ) {
                    Ok(_) => {
                        let last_id_sql = "SELECT LAST_INSERT_ID();";
                        let data: Vec<usize> = conn.query(last_id_sql).unwrap();
                        return Ok(data);
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
    use crate::{
        record::Record,
        storage::{ShardConfig, StorageAdapter},
    };

    use super::{build_mysql_conn_pool, MySQLStorageAdapter};

    #[tokio::test]
    async fn mysql_set() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let key = String::from("name");
        let value = String::from("loboxu");
        mysql_adapter
            .set(key.clone(), Record::build_e(value.clone()))
            .await
            .unwrap();

        assert!(mysql_adapter.exists(key.clone()).await.unwrap());

        let get_res = mysql_adapter.get(key.clone()).await.unwrap().unwrap();
        assert_eq!(String::from_utf8(get_res.data).unwrap(), value.clone());

        mysql_adapter.delete(key.clone()).await.unwrap();

        assert!(!mysql_adapter.exists(key.clone()).await.unwrap());
    }

    #[tokio::test]
    async fn mysql_create_table() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let shard_name = String::from("test");
        let shard_config = ShardConfig::default();
        mysql_adapter
            .create_shard(shard_name, shard_config)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn mysql_drop_table() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let shard_name = String::from("test");
        mysql_adapter.delete_shard(shard_name).await.unwrap();
    }

    #[tokio::test]
    async fn mysql_stream_write() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let shard_name = String::from("test");
        let shard_config = ShardConfig::default();
        mysql_adapter
            .create_shard(shard_name.clone(), shard_config)
            .await
            .unwrap();
        let mut data = Vec::new();
        data.push(Record::build_e("test1".to_string()));
        data.push(Record::build_e("test2".to_string()));
        let result = mysql_adapter.stream_write(shard_name, data).await.unwrap();
        println!("{:?}", result);
    }
}
