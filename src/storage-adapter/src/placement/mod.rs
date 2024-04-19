use crate::{record::Record, storage::StorageAdapter};
use axum::async_trait;
use clients::{
    placement_center::kv::{placement_delete, placement_exists, placement_get, placement_set},
    ClientPool,
};
use common_base::errors::RobustMQError;
use protocol::placement_center::generate::kv::{
    DeleteRequest, ExistsRequest, GetRequest, SetRequest,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct PlacementStorageAdapter {
    client_poll: Arc<Mutex<ClientPool>>,
    addr: String,
}

impl PlacementStorageAdapter {
    pub fn new(client_poll: Arc<Mutex<ClientPool>>, addr: String) -> Self {
        return PlacementStorageAdapter { client_poll, addr };
    }
}

#[async_trait]
impl StorageAdapter for PlacementStorageAdapter {
    async fn set(&self, key: String, value: Record) -> Result<(), RobustMQError> {
        let request = SetRequest {
            key,
            value: String::from_utf8(value.data).unwrap(),
        };
        match placement_set(self.client_poll.clone(), self.addr.clone(), request).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    async fn get(&self, key: String) -> Result<Option<Record>, RobustMQError> {
        let request = GetRequest { key };
        match placement_get(self.client_poll.clone(), self.addr.clone(), request).await {
            Ok(reply) => return Ok(Some(Record::build_e(reply.value))),
            Err(e) => {
                return Err(e);
            }
        }
    }
    async fn delete(&self, key: String) -> Result<(), RobustMQError> {
        let request = DeleteRequest { key };
        match placement_delete(self.client_poll.clone(), self.addr.clone(), request).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                return Err(e);
            }
        }
    }
    async fn exists(&self, key: String) -> Result<bool, RobustMQError> {
        let request = ExistsRequest { key };
        match placement_exists(self.client_poll.clone(), self.addr.clone(), request).await {
            Ok(reply) => return Ok(reply.flag),
            Err(e) => {
                return Err(e);
            }
        }
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
