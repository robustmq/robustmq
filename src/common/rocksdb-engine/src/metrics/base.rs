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

use common_base::{
    error::common::CommonError,
    tools::now_second,
};
use dashmap::DashMap;
use std::sync::Arc;

use crate::metrics::expire::{DB_COLUMN_FAMILY_METRICS, DB_COLUMN_FAMILY_METRICS_PRE};
use crate::metrics::MetricsValue;
use crate::storage::broker::{engine_prefix_list_by_broker, engine_save_by_broker};
use crate::{rocksdb::RocksDBEngine, storage::broker::engine_get_by_broker};

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
    engine_save_by_broker(rocksdb_engine, &db_key, value)
}

pub(crate) fn get_metric_data(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key_prefix: &str,
) -> Result<DashMap<u64, u64>, CommonError> {
    let prefix = format!("{}/{}/", DB_COLUMN_FAMILY_METRICS, key_prefix);
    let results = DashMap::new();
    for row in engine_prefix_list_by_broker::<MetricsValue>(rocksdb_engine, &prefix)? {
        results.insert(row.data.timestamp, row.data.value);
    }

    Ok(results)
}

pub(crate) fn record_pre_num(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key: &str,
    total: u64,
) -> Result<(), CommonError> {
    let db_key = format!("{}/{}", DB_COLUMN_FAMILY_METRICS_PRE, key);
    let value = MetricsValue::new(total, now_second());
    engine_save_by_broker(rocksdb_engine, &db_key, value)
}

pub(crate) async fn get_pre_num(
    rocksdb_engine: &Arc<RocksDBEngine>,
    key: &str,
) -> Result<u64, CommonError> {
    let db_key = format!("{}/{}", DB_COLUMN_FAMILY_METRICS_PRE, key);
    let res = match engine_get_by_broker::<MetricsValue>(rocksdb_engine, &db_key)? {
        Some(data) => data,
        None => return Ok(0),
    };
    Ok(res.data.value)
}
#[macro_export]
macro_rules! define_simple_metric {
    ($record_fn:ident, $get_fn:ident, $key:expr) => {
        pub fn $record_fn(&self, time: u64, num: u64) -> Result<(), CommonError> {
            $crate::metrics::base::record_num(&self.rocksdb_engine, $key, time, num)
        }

        pub fn $get_fn(&self) -> Result<DashMap<u64, u64>, CommonError> {
            $crate::metrics::base::get_metric_data(&self.rocksdb_engine, $key)
        }
    };
}

#[macro_export]
macro_rules! define_cumulative_metric {
    ($record_fn:ident, $get_fn:ident, $get_pre_fn:ident, $get_rate_fn:ident, $key:expr) => {
        pub async fn $record_fn(&self, time: u64, total: u64, num: u64) -> Result<(), CommonError> {
            $crate::metrics::base::record_num(&self.rocksdb_engine, $key, time, num)?;
            $crate::metrics::base::record_pre_num(&self.rocksdb_engine, $key, total)
        }

        pub fn $get_fn(&self) -> Result<DashMap<u64, u64>, CommonError> {
            $crate::metrics::base::get_metric_data(&self.rocksdb_engine, $key)
        }

        pub async fn $get_pre_fn(&self) -> Result<u64, CommonError> {
            $crate::metrics::base::get_pre_num(&self.rocksdb_engine, $key).await
        }

        pub fn $get_rate_fn(&self) -> Result<u64, CommonError> {
            use $crate::metrics::get_max_key_value;
            let data = self.$get_fn()?;
            Ok(get_max_key_value(&data))
        }
    };
}

#[macro_export]
macro_rules! define_dimensional_metric_1d {
    ($record_fn:ident, $get_fn:ident, $get_pre_fn:ident, $key:expr, $dim1:ident: $dim1_ty:ty) => {
        pub fn $record_fn(
            &self,
            $dim1: $dim1_ty,
            time: u64,
            total: u64,
            num: u64,
        ) -> Result<(), CommonError> {
            let key = format!("{}_{}", $key, $dim1);
            $crate::metrics::base::record_num(&self.rocksdb_engine, &key, time, num)?;
            $crate::metrics::base::record_pre_num(&self.rocksdb_engine, &key, total)
        }

        pub fn $get_fn(&self, $dim1: $dim1_ty) -> Result<DashMap<u64, u64>, CommonError> {
            let key = format!("{}_{}", $key, $dim1);
            $crate::metrics::base::get_metric_data(&self.rocksdb_engine, &key)
        }

        pub async fn $get_pre_fn(&self, $dim1: $dim1_ty, num: u64) -> Result<u64, CommonError> {
            let key = format!("{}_{}", $key, $dim1);
            Ok(
                $crate::metrics::base::get_pre_num(&self.rocksdb_engine, &key)
                    .await
                    .map_or(num, |v| v),
            )
        }
    };
}

#[macro_export]
macro_rules! define_dimensional_metric_3d {
    ($record_fn:ident, $get_fn:ident, $get_pre_fn:ident, $key:expr,
     $dim1:ident: $dim1_ty:ty, $dim2:ident: $dim2_ty:ty, $dim3:ident: $dim3_ty:ty) => {
        pub fn $record_fn(
            &self,
            $dim1: $dim1_ty,
            $dim2: $dim2_ty,
            $dim3: $dim3_ty,
            time: u64,
            total: u64,
            num: u64,
        ) -> Result<(), CommonError> {
            let key = format!("{}_{}_{}_{}", $key, $dim1, $dim2, $dim3);
            $crate::metrics::base::record_num(&self.rocksdb_engine, &key, time, num)?;
            $crate::metrics::base::record_pre_num(&self.rocksdb_engine, &key, total)
        }

        pub fn $get_fn(
            &self,
            $dim1: $dim1_ty,
            $dim2: $dim2_ty,
            $dim3: $dim3_ty,
        ) -> Result<DashMap<u64, u64>, CommonError> {
            let key = format!("{}_{}_{}_{}", $key, $dim1, $dim2, $dim3);
            $crate::metrics::base::get_metric_data(&self.rocksdb_engine, &key)
        }

        pub async fn $get_pre_fn(
            &self,
            $dim1: $dim1_ty,
            $dim2: $dim2_ty,
            $dim3: $dim3_ty,
            num: u64,
        ) -> Result<u64, CommonError> {
            let key = format!("{}_{}_{}_{}", $key, $dim1, $dim2, $dim3);
            Ok(
                $crate::metrics::base::get_pre_num(&self.rocksdb_engine, &key)
                    .await
                    .map_or(num, |v| v),
            )
        }
    };
}

#[macro_export]
macro_rules! define_dimensional_metric_4d {
    ($record_fn:ident, $get_fn:ident, $get_pre_fn:ident, $key:expr,
     $dim1:ident: $dim1_ty:ty, $dim2:ident: $dim2_ty:ty, $dim3:ident: $dim3_ty:ty, $dim4:ident: $dim4_ty:ty) => {
        #[allow(clippy::too_many_arguments)]
        pub fn $record_fn(
            &self,
            $dim1: $dim1_ty,
            $dim2: $dim2_ty,
            $dim3: $dim3_ty,
            $dim4: $dim4_ty,
            time: u64,
            total: u64,
            num: u64,
        ) -> Result<(), CommonError> {
            let key = format!("{}_{}_{}_{}_{}", $key, $dim1, $dim2, $dim3, $dim4);
            $crate::metrics::base::record_num(&self.rocksdb_engine, &key, time, num)?;
            $crate::metrics::base::record_pre_num(&self.rocksdb_engine, &key, total)
        }

        pub fn $get_fn(
            &self,
            $dim1: $dim1_ty,
            $dim2: $dim2_ty,
            $dim3: $dim3_ty,
            $dim4: $dim4_ty,
        ) -> Result<DashMap<u64, u64>, CommonError> {
            let key = format!("{}_{}_{}_{}_{}", $key, $dim1, $dim2, $dim3, $dim4);
            $crate::metrics::base::get_metric_data(&self.rocksdb_engine, &key)
        }

        pub async fn $get_pre_fn(
            &self,
            $dim1: $dim1_ty,
            $dim2: $dim2_ty,
            $dim3: $dim3_ty,
            $dim4: $dim4_ty,
            num: u64,
        ) -> Result<u64, CommonError> {
            let key = format!("{}_{}_{}_{}_{}", $key, $dim1, $dim2, $dim3, $dim4);
            Ok(
                $crate::metrics::base::get_pre_num(&self.rocksdb_engine, &key)
                    .await
                    .map_or(num, |v| v),
            )
        }
    };
}

#[cfg(test)]
mod tests {
    use common_base::{tools::now_second, uuid::unique_id};

    use crate::{
        metrics::base::{get_metric_data, get_pre_num, record_num, record_pre_num},
        test::test_rocksdb_instance,
    };

    #[tokio::test]
    async fn base_test() {
        let rs_handler = test_rocksdb_instance();
        let key = unique_id();
        let time = now_second();
        let num = 100;
        let res = record_num(&rs_handler, &key, time, num);
        assert!(res.is_ok());

        let data = get_metric_data(&rs_handler, &key).unwrap();
        assert_eq!(data.len(), 1);

        record_pre_num(&rs_handler, &key, 100).unwrap();
        let res = get_pre_num(&rs_handler, &key).await.unwrap();
        assert_eq!(res, 100);
    }
}
