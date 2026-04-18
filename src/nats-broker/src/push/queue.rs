use std::sync::Arc;

use broker_core::share_group::ShareGroupStorage;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::{mqtt::share_group::ShareGroupMember, nats::subscribe::NatsSubscribe};

pub async fn add_member_by_group(client_pool: Arc<ClientPool>, sub: &NatsSubscribe) {
    let conf = broker_config();
    let storage = ShareGroupStorage::new(client_pool);
    let member = ShareGroupMember {
        broker_id: conf.broker_id,
        connect_id: sub.connect_id,
    };
    if let Err(e) = storage.add_member(&sub.tenant, &sub.queue_group, &member).await {
        tracing::warn!(
            "Failed to add group member: tenant={}, group={}, error={}",
            sub.tenant, sub.queue_group, e
        );
    }
}

pub async fn remove_member_by_connect_id(_connect_id: u64) {}

pub async fn remove_member_by_group(_connect_id: u64, _tenant: &str, _group: &str) {}
