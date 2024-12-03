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

use axum::async_trait;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use metadata_struct::adapter::record::{Header, Record};
use mysql::prelude::Queryable;
use mysql::{params, Pool};

use self::schema::{TMqttKvMsg, TMqttRecord};
use crate::storage::{ShardConfig, StorageAdapter};
pub mod schema;

#[derive(Clone)]
pub struct MySQLStorageAdapter {
    pool: Pool,
}

impl MySQLStorageAdapter {
    pub fn new(pool: Pool) -> Self {
        let adapter = MySQLStorageAdapter { pool };
        match adapter.init_table() {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e.to_string())
            }
        }
        adapter
    }

    pub fn storage_record_table(&self, shard_name: String) -> String {
        format!("storage_record_{}", shard_name)
    }

    pub fn storage_kv_table(&self) -> String {
        "storage_kv".to_string()
    }

    pub fn group_offset_key(&self, shard_name: String, group_name: String) -> String {
        format!("__group_offset_{}_{}", group_name, shard_name)
    }

    pub fn init_table(&self) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let show_table_sql = "
                CREATE TABLE IF NOT EXISTS `storage_kv` (
                    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
                    `data_key` varchar(128) DEFAULT NULL,
                    `data_value` blob,
                    `create_time` int(11) NOT NULL,
                    `update_time` int(11) NOT NULL,
                    PRIMARY KEY (`id`),
                    unique index data_key(data_key)
                    ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;
                ";
                match conn.query_drop(show_table_sql) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(CommonError::CommonError(e.to_string())),
                }
            }
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn set(&self, key: String, value: Record) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let values = [TMqttKvMsg {
                    key,
                    value: value.data,
                    create_time: now_second(),
                    update_time: now_second(),
                }];
                match conn.exec_batch(
                    format!("REPLACE INTO {}(data_key,data_value,create_time,update_time) VALUES (:key,:value,:create_time,:update_time)",self.storage_kv_table()),
                    values.iter().map(|p| {
                        params! {
                            "key" => p.key.clone(),
                            "value" => p.value.clone(),
                            "create_time" => p.create_time,
                            "update_time" => p.update_time,
                        }
                    }),
                ) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(CommonError::CommonError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn get(&self, key: String) -> Result<Option<Record>, CommonError> {
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
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn delete(&self, key: String) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "delete from {} where data_key='{}'",
                    self.storage_kv_table(),
                    key
                );
                match conn.query_drop(sql) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(CommonError::CommonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn exists(&self, key: String) -> Result<bool, CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "select count(*) as count from {} where data_key='{}'",
                    self.storage_kv_table(),
                    key
                );
                let data: Vec<u32> = conn.query(sql).unwrap();
                if let Some(value) = data.first() {
                    return Ok(*value > 0);
                }
                return Ok(false);
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }
}

#[async_trait]
impl StorageAdapter for MySQLStorageAdapter {
    async fn create_shard(&self, shard_name: String, _: ShardConfig) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let show_table_sql = format!(
                    "CREATE TABLE IF NOT EXISTS `{}` (
                    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
                    `msgid` varchar(64) DEFAULT NULL,
                    `header` text DEFAULT NULL,
                    `msg_key` varchar(128) DEFAULT NULL,
                    `payload` blob,
                    `create_time` int(11) NOT NULL,
                    PRIMARY KEY (`id`)
                    ) ENGINE=InnoDB DEFAULT CHARSET=utf8MB4;",
                    self.storage_record_table(shard_name)
                );
                match conn.query_drop(show_table_sql) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(CommonError::CommonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn delete_shard(&self, shard_name: String) -> Result<(), CommonError> {
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let show_table_sql = format!(
                    "DROP TABLE IF EXISTS `{}`",
                    self.storage_record_table(shard_name)
                );
                match conn.query_drop(show_table_sql) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(CommonError::CommonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn stream_write(
        &self,
        shard_name: String,
        data: Vec<Record>,
    ) -> Result<Vec<usize>, CommonError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let mut values = Vec::new();
                for raw in data {
                    values.push(TMqttRecord {
                        msgid: String::from(""),
                        header: if let Some(h) = raw.header {
                            serde_json::to_string(&h).unwrap()
                        } else {
                            "".to_string()
                        },
                        msg_key: if let Some(k) = raw.key {
                            k
                        } else {
                            "".to_string()
                        },
                        payload: raw.data,
                        create_time: now_second(),
                    });
                }

                match conn.exec_batch(
                    format!("INSERT INTO {}(msgid,header,msg_key,payload,create_time) VALUES (:msgid,:header,:msg_key,:payload,:create_time)",self.storage_record_table(shard_name)),
                    values.iter().map(|p| {
                        params! {
                            "msgid" => p.msgid.clone(),
                            "header" => p.header.clone(),
                            "msg_key" => p.msg_key.clone(),
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
                        return Err(CommonError::CommonError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn stream_read(
        &self,
        shard_name: String,
        group_id: String,
        record_num: Option<u128>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        let offset_key = self.group_offset_key(shard_name.clone(), group_id);
        let offset = match self.get(offset_key).await {
            Ok(Some(record)) => {
                let offset_str = String::from_utf8(record.data).unwrap();
                offset_str.parse::<usize>().unwrap()
            }
            Ok(None) => 0,
            Err(e) => return Err(CommonError::CommonError(e.to_string())),
        };
        let rn = record_num.unwrap_or(10);
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let sql = format!(
                    "select id,msgid,header,msg_key,payload,create_time from {} where id > {} limit {}",
                    self.storage_record_table(shard_name),
                    offset,
                    rn
                );
                let data: Vec<(usize, String, String, String, Vec<u8>, usize)> =
                    conn.query(sql).unwrap();
                let mut result = Vec::new();
                for raw in data {
                    let headers: Vec<Header> =
                        serde_json::from_str::<Vec<Header>>(&raw.2).unwrap_or_default();
                    result.push(Record {
                        offset: raw.0 as u128,
                        header: Some(headers),
                        key: Some(raw.3),
                        data: raw.4,
                        create_time: Some(raw.5 as u128),
                    })
                }
                return Ok(Some(result));
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn stream_commit_offset(
        &self,
        shard_name: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, CommonError> {
        // batch update offset
        match self.pool.get_conn() {
            Ok(mut conn) => {
                let update_sql = format!(
                    "REPLACE INTO {}(data_key,data_value,create_time,update_time) VALUES ('{}','{:?}',{},{})",
                    self.storage_kv_table(),
                    self.group_offset_key(shard_name, group_id),
                    offset,
                    now_second(),
                    now_second(),
                );

                match conn.query_drop(update_sql) {
                    Ok(()) => return Ok(true),
                    Err(e) => return Err(CommonError::CommonError(e.to_string())),
                }
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn stream_read_by_offset(
        &self,
        _: String,
        _: usize,
    ) -> Result<Option<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
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
    ) -> Result<Option<Vec<Record>>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_key(
        &self,
        _: String,
        _: String,
    ) -> Result<Option<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "PlacementStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use metadata_struct::adapter::record::{Header, Record};
    use third_driver::mysql::build_mysql_conn_pool;

    use super::MySQLStorageAdapter;
    use crate::storage::{ShardConfig, StorageAdapter};

    #[tokio::test]
    #[ignore]
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

        // mysql_adapter.delete(key.clone()).await.unwrap();

        // assert!(!mysql_adapter.exists(key.clone()).await.unwrap());
    }

    #[tokio::test]
    #[ignore]
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
    #[ignore]
    async fn mysql_drop_table() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let shard_name = String::from("test");
        mysql_adapter.delete_shard(shard_name).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
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
        let header = vec![Header {
            name: "n1".to_string(),
            value: "v1".to_string(),
        }];
        data.push(Record::build_d(
            "k1".to_string(),
            header.clone(),
            "test1".to_string().as_bytes().to_vec(),
        ));
        data.push(Record::build_d(
            "k2".to_string(),
            header,
            "test2".to_string().as_bytes().to_vec(),
        ));
        let result = mysql_adapter.stream_write(shard_name, data).await.unwrap();
        println!("{:?}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn mysql_stream_read_and_commit() {
        let addr = "mysql://root:123456@127.0.0.1:3306/mqtt";
        let pool = build_mysql_conn_pool(addr).unwrap();
        let mysql_adapter = MySQLStorageAdapter::new(pool);
        let shard_name = String::from("test");
        let group_id = String::from("test-group");
        let record_num = 2;
        let record_size = 1024;
        let result = mysql_adapter
            .stream_read(
                shard_name.clone(),
                group_id.clone(),
                Some(record_num),
                Some(record_size),
            )
            .await
            .unwrap()
            .unwrap();
        if !result.is_empty() {
            let offset = result.last().unwrap().offset;
            mysql_adapter
                .stream_commit_offset(shard_name, group_id, offset)
                .await
                .unwrap();
        }
    }
}
