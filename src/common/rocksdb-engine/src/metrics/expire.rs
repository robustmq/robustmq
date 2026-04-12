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

use crate::metrics::MetricsValue;
use crate::rocksdb::RocksDBEngine;
use crate::storage::broker::{engine_delete_by_broker, engine_delete_prefix_by_broker};
use crate::storage::family::DB_COLUMN_FAMILY_BROKER;
use crate::warp::StorageDataWrap;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_base::utils::serialize;
use std::sync::Arc;
use tokio::sync::broadcast;

pub(crate) const DB_COLUMN_FAMILY_METRICS: &str = "/metrics/data";
pub(crate) const DB_COLUMN_FAMILY_METRICS_PRE: &str = "/metrics/pre_num";
const DB_COLUMN_FAMILY_METRICS_PREFIX: &str = "/metrics/data/";
const DB_COLUMN_FAMILY_METRICS_PRE_PREFIX: &str = "/metrics/pre_num/";

// Scan every hour; metrics data is low-churn so frequent GC adds no value.
const METRICS_GC_INTERVAL_MS: u64 = 60 * 60 * 1000;

// Default retention: 7 days.
pub const METRICS_DEFAULT_EXPIRE_SEC: u64 = 7 * 24 * 3600;

pub async fn start_metrics_gc_thread(
    rocksdb_engine: Arc<RocksDBEngine>,
    expire_sec: u64,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        gc(&rocksdb_engine, expire_sec).map_err(|e| CommonError::CommonError(e.to_string()))?;
        Ok(())
    };
    loop_select_ticket(ac_fn, METRICS_GC_INTERVAL_MS, &stop_send).await;
}

pub fn gc(rocksdb_engine: &Arc<RocksDBEngine>, save_time: u64) -> Result<(), CommonError> {
    gc_prefix(rocksdb_engine, DB_COLUMN_FAMILY_METRICS_PREFIX, save_time)?;
    gc_prefix(
        rocksdb_engine,
        DB_COLUMN_FAMILY_METRICS_PRE_PREFIX,
        save_time,
    )?;
    Ok(())
}

fn gc_prefix(
    rocksdb_engine: &Arc<RocksDBEngine>,
    prefix: &str,
    save_time: u64,
) -> Result<(), CommonError> {
    let now_time = now_second();

    let cf = if let Some(cf) = rocksdb_engine.cf_handle(DB_COLUMN_FAMILY_BROKER) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            DB_COLUMN_FAMILY_BROKER.to_string(),
        ));
    };
    let mut iter = rocksdb_engine.db.raw_iterator_cf(&cf);
    iter.seek(prefix);

    while iter.valid() {
        if let Some(key) = iter.key() {
            if let Some(val) = iter.value() {
                let key = String::from_utf8(key.to_vec())?;
                if !key.starts_with(prefix) {
                    break;
                }
                let value = val.to_vec();
                if let Ok(v) =
                    serialize::deserialize::<StorageDataWrap<MetricsValue>>(value.as_ref())
                {
                    if now_time > v.create_time.saturating_add(save_time) {
                        engine_delete_by_broker(rocksdb_engine, &key)?;
                    }
                }
            }
        }
        iter.next();
    }

    Ok(())
}

pub fn delete_by_prefix(
    rocksdb_engine: &Arc<RocksDBEngine>,
    prefix_key: &str,
) -> ResultCommonError {
    let key = format!("{}/{}", DB_COLUMN_FAMILY_METRICS, prefix_key);
    engine_delete_prefix_by_broker(rocksdb_engine, &key)?;

    let key = format!("{}/{}", DB_COLUMN_FAMILY_METRICS_PRE, prefix_key);
    engine_delete_prefix_by_broker(rocksdb_engine, &key)?;
    Ok(())
}
