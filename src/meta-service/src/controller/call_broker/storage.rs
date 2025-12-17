use std::sync::Arc;

use grpc_clients::pool::ClientPool;
use metadata_struct::journal::{
    segment::JournalSegment, segment_meta::JournalSegmentMetadata, shard::JournalShard,
};
use protocol::broker::broker_mqtt::{
    MqttBrokerUpdateCacheActionType, MqttBrokerUpdateCacheResourceType,
};

use crate::{
    controller::call_broker::{
        call::add_call_message,
        mqtt::{BrokerCallManager, BrokerCallMessage},
    },
    core::error::MetaServiceError,
};

pub async fn update_cache_by_set_shard(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_info: JournalShard,
) -> Result<(), MetaServiceError> {
    let data = shard_info.encode()?;
    let message = BrokerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Shard,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_segment(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment_info: JournalSegment,
) -> Result<(), MetaServiceError> {
    let data = segment_info.encode()?;
    let message = BrokerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::Segment,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}

pub async fn update_cache_by_set_segment_meta(
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment_info: JournalSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = segment_info.encode()?;
    let message = BrokerCallMessage {
        action_type: MqttBrokerUpdateCacheActionType::Set,
        resource_type: MqttBrokerUpdateCacheResourceType::SegmentMeta,
        data,
    };
    add_call_message(call_manager, client_pool, message).await?;
    Ok(())
}
