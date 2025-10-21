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
use common_base::tools::now_second;
use dashmap::DashMap;
use std::sync::Arc;

use crate::metrics_cache::MetricsValue;
use crate::storage::broker::{
    engine_delete_by_broker, engine_prefix_list_by_broker, engine_save_by_broker,
};
use crate::storage::family::DB_COLUMN_FAMILY_BROKER;
use crate::warp::StorageDataWrap;
use crate::{rocksdb::RocksDBEngine, storage::broker::engine_get_by_broker};

const DB_COLUMN_FAMILY_METRICS: &str = "/metrics/data";
const DB_COLUMN_FAMILY_METRICS_PRE: &str = "/metrics/pre_num";
const DB_COLUMN_FAMILY_METRICS_PREFIX: &str = "/metrics/";

pub(crate) fn record_num(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key: &str,
    time: u64,
    num: u64,
) -> Result<(), CommonError> {
    let db_key = format!("{}/{}/{}", DB_COLUMN_FAMILY_METRICS, key, time);
    let value = MetricsValue {
        value: num,
        timestamp: time,
    };
    engine_save_by_broker(rocksdb_engine.clone(), db_key, value)
}

pub(crate) fn get_metric_data(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key_prefix: &str,
) -> Result<DashMap<u64, u64>, CommonError> {
    let prefix = format!("{}/{}/", DB_COLUMN_FAMILY_METRICS, key_prefix);
    let results = DashMap::new();
    for row in engine_prefix_list_by_broker(rocksdb_engine.clone(), prefix)? {
        let data = serde_json::from_str::<MetricsValue>(&row.data)?;
        results.insert(data.timestamp, data.value);
    }

    Ok(results)
}

pub(crate) fn record_pre_num(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key: &str,
    total: u64,
) -> Result<(), CommonError> {
    let db_key = format!("{}/{}", DB_COLUMN_FAMILY_METRICS_PRE, key);
    engine_save_by_broker(rocksdb_engine.clone(), db_key, total)
}

pub(crate) async fn get_pre_num(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key: &str,
) -> Result<u64, CommonError> {
    let db_key = format!("{}/{}", DB_COLUMN_FAMILY_METRICS_PRE, key);
    let res = match engine_get_by_broker(rocksdb_engine.clone(), db_key)? {
        Some(data) => data,
        None => return Ok(0),
    };
    Ok(serde_json::from_str::<u64>(&res.data)?)
}

pub fn gc(rocksdb_engine: &Arc<RocksDBEngine>) -> Result<(), CommonError> {
    let now_time = now_second();
    let save_time = 3600;

    let cf = if let Some(cf) = rocksdb_engine.cf_handle(DB_COLUMN_FAMILY_BROKER) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            DB_COLUMN_FAMILY_BROKER.to_string(),
        ));
    };
    let mut iter = rocksdb_engine.db.raw_iterator_cf(&cf);
    iter.seek(DB_COLUMN_FAMILY_METRICS_PREFIX);

    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(DB_COLUMN_FAMILY_METRICS_PREFIX) {
                    break;
                }
                let value = val.to_vec();
                match serde_json::from_slice::<StorageDataWrap>(value.as_ref()) {
                    Ok(v) => {
                        if now_time > (v.create_time + save_time) {
                            engine_delete_by_broker(rocksdb_engine.clone(), key)?;
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }

        iter.next();
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
