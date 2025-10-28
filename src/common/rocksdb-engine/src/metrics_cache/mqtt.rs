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

use super::get_max_key_value;
use crate::{
    metrics_cache::base::{
        delete_by_prefix, get_metric_data, get_pre_num, record_num, record_pre_num,
    },
    rocksdb::RocksDBEngine,
};
use common_base::error::{common::CommonError, ResultCommonError};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

pub const METRICS_TYPE_KEY_CONNECTION_NUM: &str = "connection_num";
pub const METRICS_TYPE_KEY_TOPIC_NUM: &str = "topic_num";
pub const METRICS_TYPE_KEY_SUBSCRIBE_NUM: &str = "subscribe_num";
pub const METRICS_TYPE_KEY_MESSAGE_IN_NUM: &str = "message_in";
pub const METRICS_TYPE_KEY_MESSAGE_OUT_NUM: &str = "message_out";
pub const METRICS_TYPE_KEY_MESSAGE_DROP_NUM: &str = "message_drop";
pub const METRICS_TYPE_KEY_TOPIC_IN_NUM: &str = "topic_in";
pub const METRICS_TYPE_KEY_TOPIC_OUT_NUM: &str = "topic_out";
pub const METRICS_TYPE_KEY_SUBSCRIBE_SEND: &str = "subscribe_send";
pub const METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND: &str = "subscribe_topic_send";
pub const METRICS_TYPE_KEY_SESSION_IN_NUM: &str = "session_in";
pub const METRICS_TYPE_KEY_SESSION_OUT_NUM: &str = "session_out";

#[derive(Clone)]
pub struct MQTTMetricsCache {
    rocksdb_engine: Arc<RocksDBEngine>,
}

impl MQTTMetricsCache {
    pub fn new(rocksdb_engine: Arc<RocksDBEngine>) -> Self {
        Self { rocksdb_engine }
    }

    // Connection num
    pub fn record_connection_num(&self, time: u64, num: u64) -> Result<(), CommonError> {
        record_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_CONNECTION_NUM,
            time,
            num,
        )
    }

    pub fn get_connection_num(&self) -> Result<DashMap<u64, u64>, CommonError> {
        get_metric_data(&self.rocksdb_engine, METRICS_TYPE_KEY_CONNECTION_NUM)
    }

    // Topic num
    pub fn record_topic_num(&self, time: u64, num: u64) -> Result<(), CommonError> {
        record_num(&self.rocksdb_engine, METRICS_TYPE_KEY_TOPIC_NUM, time, num)
    }

    pub fn get_topic_num(&self) -> Result<DashMap<u64, u64>, CommonError> {
        get_metric_data(&self.rocksdb_engine, METRICS_TYPE_KEY_TOPIC_NUM)
    }

    // topic in
    pub fn record_topic_in_num(
        &self,
        topic: &str,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        record_num(&self.rocksdb_engine, &key, time, num)?;
        record_pre_num(&self.rocksdb_engine, &key, total)
    }

    pub fn get_topic_in_num(&self, topic: &str) -> Result<DashMap<u64, u64>, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        get_metric_data(&self.rocksdb_engine, &key)
    }

    pub async fn get_topic_in_pre_total(&self, topic: &str, num: u64) -> Result<u64, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        Ok(get_pre_num(&self.rocksdb_engine, &key)
            .await
            .map_or(num, |v| v))
    }

    // topic out
    pub fn record_topic_out_num(
        &self,
        topic: &str,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        record_num(&self.rocksdb_engine, &key, time, num)?;
        record_pre_num(&self.rocksdb_engine, &key, total)
    }

    pub fn get_topic_out_num(&self, topic: &str) -> Result<DashMap<u64, u64>, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        get_metric_data(&self.rocksdb_engine, &key)
    }

    pub async fn get_topic_out_pre_total(&self, topic: &str, num: u64) -> Result<u64, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        Ok(get_pre_num(&self.rocksdb_engine, &key)
            .await
            .map_or(num, |v| v))
    }

    // Subscribe num
    pub fn record_subscribe_num(&self, time: u64, num: u64) -> Result<(), CommonError> {
        record_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_SUBSCRIBE_NUM,
            time,
            num,
        )
    }

    pub fn get_subscribe_num(&self) -> Result<DashMap<u64, u64>, CommonError> {
        get_metric_data(&self.rocksdb_engine, METRICS_TYPE_KEY_SUBSCRIBE_NUM)
    }

    // Subscribe send num
    pub fn record_subscribe_send_num(
        &self,
        client_id: &str,
        path: &str,
        success: bool,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        let key = format!(
            "{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path, success
        );
        record_num(&self.rocksdb_engine, &key, time, num)?;
        record_pre_num(&self.rocksdb_engine, &key, total)
    }

    pub fn get_subscribe_send_num(
        &self,
        client_id: &str,
        path: &str,
        success: bool,
    ) -> Result<DashMap<u64, u64>, CommonError> {
        let key = format!(
            "{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path, success
        );
        get_metric_data(&self.rocksdb_engine, &key)
    }

    pub async fn get_subscribe_send_pre_total(
        &self,
        client_id: &str,
        path: &str,
        success: bool,
        num: u64,
    ) -> Result<u64, CommonError> {
        let key = format!(
            "{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path, success
        );
        Ok(get_pre_num(&self.rocksdb_engine, &key)
            .await
            .map_or(num, |v| v))
    }

    // subscribe topic
    #[allow(clippy::too_many_arguments)]
    pub fn record_subscribe_topic_send_num(
        &self,
        client_id: &str,
        path: &str,
        topic: &str,
        success: bool,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        let key = format!(
            "{}_{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path, topic, success
        );
        record_num(&self.rocksdb_engine, &key, time, num)?;
        record_pre_num(&self.rocksdb_engine, &key, total)
    }

    pub fn get_subscribe_topic_send_num(
        &self,
        client_id: &str,
        path: &str,
        topic: &str,
        success: bool,
    ) -> Result<DashMap<u64, u64>, CommonError> {
        let key = format!(
            "{}_{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path, topic, success
        );
        get_metric_data(&self.rocksdb_engine, &key)
    }

    pub async fn get_subscribe_topic_send_pre_total(
        &self,
        client_id: &str,
        path: &str,
        topic: &str,
        success: bool,
        num: u64,
    ) -> Result<u64, CommonError> {
        let key = format!(
            "{}_{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path, topic, success
        );
        Ok(get_pre_num(&self.rocksdb_engine, &key)
            .await
            .map_or(num, |v| v))
    }

    // message in
    pub async fn record_message_in_num(
        &self,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        record_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_MESSAGE_IN_NUM,
            time,
            num,
        )?;
        record_pre_num(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_IN_NUM, total)
    }

    pub fn get_message_in_num(&self) -> Result<DashMap<u64, u64>, CommonError> {
        get_metric_data(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_IN_NUM)
    }

    pub async fn get_pre_message_in(&self) -> Result<u64, CommonError> {
        get_pre_num(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_IN_NUM).await
    }

    pub fn get_message_in_rate(&self) -> Result<u64, CommonError> {
        let data = self.get_message_in_num()?;
        Ok(get_max_key_value(&data))
    }

    // message out
    pub async fn record_message_out_num(
        &self,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        record_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_MESSAGE_OUT_NUM,
            time,
            num,
        )?;
        record_pre_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_MESSAGE_OUT_NUM,
            total,
        )
    }

    pub fn get_message_out_num(&self) -> Result<DashMap<u64, u64>, CommonError> {
        get_metric_data(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_OUT_NUM)
    }

    pub async fn get_pre_message_out(&self) -> Result<u64, CommonError> {
        get_pre_num(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_OUT_NUM).await
    }

    pub fn get_message_out_rate(&self) -> Result<u64, CommonError> {
        let data = self.get_message_out_num()?;
        Ok(get_max_key_value(&data))
    }

    // message drop
    pub async fn record_message_drop_num(
        &self,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        record_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_MESSAGE_DROP_NUM,
            time,
            num,
        )?;
        record_pre_num(
            &self.rocksdb_engine,
            METRICS_TYPE_KEY_MESSAGE_DROP_NUM,
            total,
        )
    }

    pub fn get_message_drop_num(&self) -> Result<DashMap<u64, u64>, CommonError> {
        get_metric_data(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_DROP_NUM)
    }

    pub async fn get_pre_message_drop(&self) -> Result<u64, CommonError> {
        get_pre_num(&self.rocksdb_engine, METRICS_TYPE_KEY_MESSAGE_DROP_NUM).await
    }

    pub fn get_message_drop_rate(&self) -> Result<u64, CommonError> {
        let data = self.get_message_drop_num()?;
        Ok(get_max_key_value(&data))
    }

    // Session metrics
    pub fn record_session_in_num(
        &self,
        client_id: &str,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_IN_NUM, client_id);
        record_num(&self.rocksdb_engine, &key, time, num)?;
        record_pre_num(&self.rocksdb_engine, &key, total)
    }

    pub fn get_session_in_num(&self, client_id: &str) -> Result<DashMap<u64, u64>, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_IN_NUM, client_id);
        get_metric_data(&self.rocksdb_engine, &key)
    }

    pub async fn get_session_in_pre_total(
        &self,
        client_id: &str,
        num: u64,
    ) -> Result<u64, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_IN_NUM, client_id);
        Ok(get_pre_num(&self.rocksdb_engine, &key)
            .await
            .map_or(num, |v| v))
    }

    pub fn record_session_out_num(
        &self,
        client_id: &str,
        time: u64,
        total: u64,
        num: u64,
    ) -> Result<(), CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_OUT_NUM, client_id);
        record_num(&self.rocksdb_engine, &key, time, num)?;
        record_pre_num(&self.rocksdb_engine, &key, total)
    }

    pub fn get_session_out_num(&self, client_id: &str) -> Result<DashMap<u64, u64>, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_OUT_NUM, client_id);
        get_metric_data(&self.rocksdb_engine, &key)
    }

    pub async fn get_session_out_pre_total(
        &self,
        client_id: &str,
        num: u64,
    ) -> Result<u64, CommonError> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_OUT_NUM, client_id);
        Ok(get_pre_num(&self.rocksdb_engine, &key)
            .await
            .map_or(num, |v| v))
    }

    // Utility methods
    pub fn convert_monitor_data(&self, data_list: DashMap<u64, u64>) -> Vec<HashMap<String, u64>> {
        let mut results = Vec::new();
        for (time, value) in data_list {
            let mut raw = HashMap::new();
            raw.insert("date".to_string(), time);
            raw.insert("value".to_string(), value);
            results.push(raw);
        }
        results
    }

    pub fn remove_topic(&self, topic: &str) -> ResultCommonError {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        delete_by_prefix(&self.rocksdb_engine, &key)?;

        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        delete_by_prefix(&self.rocksdb_engine, &key)?;
        Ok(())
    }

    pub fn remove_subscribe(&self, client_id: &str, path: &str) -> ResultCommonError {
        let key = format!("{}_{}_{}", METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path);
        delete_by_prefix(&self.rocksdb_engine, &key)?;

        let key = format!(
            "{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path
        );
        delete_by_prefix(&self.rocksdb_engine, &key)?;
        Ok(())
    }

    pub fn remove_session(&self, client_id: &str) -> ResultCommonError {
        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_IN_NUM, client_id);
        delete_by_prefix(&self.rocksdb_engine, &key)?;

        let key = format!("{}_{}", METRICS_TYPE_KEY_SESSION_OUT_NUM, client_id);
        delete_by_prefix(&self.rocksdb_engine, &key)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common_base::tools::now_second;

    use crate::{metrics_cache::mqtt::MQTTMetricsCache, test::test_rocksdb_instance};

    #[tokio::test]
    async fn connection_num_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        cache.record_connection_num(time, 100).unwrap();
        assert_eq!(cache.get_connection_num().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn topic_num_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        cache.record_topic_num(time, 50).unwrap();
        assert_eq!(cache.get_topic_num().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn subscribe_num_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        cache.record_subscribe_num(time, 200).unwrap();
        assert_eq!(cache.get_subscribe_num().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn topic_in_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        let topic = "t1".to_string();
        cache.record_topic_in_num(&topic, time, 77, 100).unwrap();
        assert_eq!(cache.get_topic_in_num(&topic).unwrap().len(), 1);
        assert_eq!(cache.get_topic_in_pre_total(&topic, 5).await.unwrap(), 77);
        assert_eq!(cache.get_topic_in_pre_total("t3", 5).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn topic_out_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        let topic = "t1".to_string();
        cache.record_topic_out_num(&topic, time, 88, 150).unwrap();
        assert_eq!(cache.get_topic_out_num(&topic).unwrap().len(), 1);
        assert_eq!(cache.get_topic_out_pre_total(&topic, 5).await.unwrap(), 88);
        assert_eq!(cache.get_topic_out_pre_total("t3", 5).await.unwrap(), 0);

        cache.remove_topic(&topic).unwrap();
        assert_eq!(cache.get_topic_out_num(&topic).unwrap().len(), 0);
        assert_eq!(cache.get_topic_out_pre_total(&topic, 5).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn message_in_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        cache.record_message_in_num(time, 1000, 200).await.unwrap();
        assert_eq!(cache.get_message_in_num().unwrap().len(), 1);
        assert_eq!(cache.get_pre_message_in().await.unwrap(), 1000);
        assert_eq!(cache.get_message_in_rate().unwrap(), 200);
    }

    #[tokio::test]
    async fn message_out_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        cache.record_message_out_num(time, 800, 150).await.unwrap();
        assert_eq!(cache.get_message_out_num().unwrap().len(), 1);
        assert_eq!(cache.get_pre_message_out().await.unwrap(), 800);
        assert_eq!(cache.get_message_out_rate().unwrap(), 150);
    }

    #[tokio::test]
    async fn message_drop_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        cache.record_message_drop_num(time, 50, 10).await.unwrap();
        assert_eq!(cache.get_message_drop_num().unwrap().len(), 1);
        assert_eq!(cache.get_pre_message_drop().await.unwrap(), 50);
        assert_eq!(cache.get_message_drop_rate().unwrap(), 10);
    }

    #[tokio::test]
    async fn session_in_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        let client_id = "client1".to_string();
        cache
            .record_session_in_num(&client_id, time, 500, 100)
            .unwrap();
        assert_eq!(cache.get_session_in_num(&client_id).unwrap().len(), 1);
        assert_eq!(
            cache.get_session_in_pre_total(&client_id, 5).await.unwrap(),
            500
        );
        assert_eq!(
            cache.get_session_in_pre_total("client3", 5).await.unwrap(),
            0
        );

        cache.remove_session(&client_id).unwrap();
        assert_eq!(cache.get_session_in_num(&client_id).unwrap().len(), 0);
        assert_eq!(
            cache.get_session_in_pre_total(&client_id, 5).await.unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn session_out_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        let client_id = "client1".to_string();
        cache
            .record_session_out_num(&client_id, time, 600, 120)
            .unwrap();
        assert_eq!(cache.get_session_out_num(&client_id).unwrap().len(), 1);
        assert_eq!(
            cache
                .get_session_out_pre_total(&client_id, 5)
                .await
                .unwrap(),
            600
        );
        assert_eq!(
            cache.get_session_out_pre_total("client3", 5).await.unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn subscribe_send_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        let client_id = "client1".to_string();
        let path = "/sensor/#".to_string();
        cache
            .record_subscribe_send_num(&client_id, &path, true, time, 300, 50)
            .unwrap();
        assert_eq!(
            cache
                .get_subscribe_send_num(&client_id, &path, true)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            cache
                .get_subscribe_send_pre_total(&client_id, &path, true, 5)
                .await
                .unwrap(),
            300
        );
        assert_eq!(
            cache
                .get_subscribe_send_pre_total("client3", &path, true, 5)
                .await
                .unwrap(),
            0
        );

        cache.remove_subscribe(&client_id, &path).unwrap();
        assert_eq!(cache.get_session_in_num(&client_id).unwrap().len(), 0);
        assert_eq!(
            cache.get_session_in_pre_total(&client_id, 5).await.unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn subscribe_topic_send_test() {
        let rs_handler = test_rocksdb_instance();
        let cache = MQTTMetricsCache::new(rs_handler);
        let time = now_second();
        let client_id = "client1".to_string();
        let path = "/sensor/#".to_string();
        let topic = "/sensor/temperature".to_string();
        cache
            .record_subscribe_topic_send_num(&client_id, &path, &topic, true, time, 250, 40)
            .unwrap();
        assert_eq!(
            cache
                .get_subscribe_topic_send_num(&client_id, &path, &topic, true)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            cache
                .get_subscribe_topic_send_pre_total(&client_id, &path, &topic, true, 5)
                .await
                .unwrap(),
            250
        );
        assert_eq!(
            cache
                .get_subscribe_topic_send_pre_total("client3", &path, &topic, true, 5)
                .await
                .unwrap(),
            0
        );
    }
}
