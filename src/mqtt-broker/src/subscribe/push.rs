use std::sync::Arc;

use super::common::{get_pkid, min_qos};
use super::meta::Subscriber;
use crate::handler::cache::{CacheManager, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::handler::sub_option::{get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local};
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::common::{
    exclusive_publish_message_qos1, exclusive_publish_message_qos2, publish_message_qos,
};
use crate::subscribe::meta::SubPublishParam;
use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{Publish, PublishProperties, QoS};
use tokio::sync::broadcast::{self, Sender};
use tracing::debug;

pub async fn build_pub_message(
    record: Record,
    group_id: &str,
    qos: &QoS,
    subscriber: &Subscriber,
    sub_ids: &[usize],
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let msg = MqttMessage::decode_record(record.clone())?;

    if is_message_expire(&msg) {
        debug!("Message dropping: message expires, is not pushed to the client, and is discarded");
        return Ok(None);
    }

    if !is_send_msg_by_bo_local(subscriber.nolocal, &subscriber.client_id, &msg.client_id) {
        debug!(
            "Message dropping: message is not pushed to the client, because the client_id is the same as the subscriber, client_id: {}, topic_id: {}",
            subscriber.client_id, subscriber.topic_id
        );
        return Ok(None);
    }

    let retain = get_retain_flag_by_retain_as_published(subscriber.preserve_retain, msg.retain);

    let mut publish = Publish {
        dup: false,
        qos: qos.to_owned(),
        pkid: get_pkid(),
        retain,
        topic: Bytes::from(subscriber.topic_name.clone()),
        payload: msg.payload,
    };

    let properties = PublishProperties {
        payload_format_indicator: msg.format_indicator,
        message_expiry_interval: Some(msg.expiry_interval as u32),
        topic_alias: None,
        response_topic: msg.response_topic,
        correlation_data: msg.correlation_data,
        user_properties: msg.user_properties,
        subscription_identifiers: sub_ids.into(),
        content_type: msg.content_type,
    };

    let pkid = get_pkid();
    publish.pkid = pkid;

    let sub_pub_param = SubPublishParam::new(
        subscriber.clone(),
        publish,
        Some(properties),
        record.timestamp as u128,
        group_id.to_string(),
        pkid,
    );
    Ok(Some(sub_pub_param))
}

pub async fn publish_data(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
    sub_pub_param: SubPublishParam,
    stop_sx: &Sender<bool>,
) -> Result<(), MqttBrokerError> {
    match sub_pub_param.publish.qos {
        QoS::AtMostOnce => {
            publish_message_qos(cache_manager, connection_manager, &sub_pub_param, stop_sx).await?;
        }

        QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            let client_id = sub_pub_param.subscribe.client_id.clone();
            let pkid: u16 = sub_pub_param.pkid;
            cache_manager.add_ack_packet(
                &client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_second(),
                },
            );

            exclusive_publish_message_qos1(
                cache_manager,
                connection_manager,
                &sub_pub_param,
                stop_sx,
                &wait_puback_sx,
            )
            .await?;

            cache_manager.remove_ack_packet(&client_id, pkid);
        }

        QoS::ExactlyOnce => {
            let (wait_ack_sx, _) = broadcast::channel(1);
            let client_id = sub_pub_param.subscribe.client_id.clone();
            let pkid = sub_pub_param.pkid;
            cache_manager.add_ack_packet(
                &client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_ack_sx.clone(),
                    create_time: now_second(),
                },
            );

            exclusive_publish_message_qos2(
                cache_manager,
                connection_manager,
                &sub_pub_param,
                stop_sx,
                &wait_ack_sx,
            )
            .await?;

            cache_manager.remove_ack_packet(&client_id, pkid);
        }
    }
    Ok(())
}

pub fn build_pub_qos(cache_manager: &Arc<CacheManager>, subscriber: &Subscriber) -> QoS {
    let cluster_qos = cache_manager.get_cluster_info().protocol.max_qos;
    min_qos(cluster_qos, subscriber.qos)
}

pub fn build_sub_ids(subscriber: &Subscriber) -> Vec<usize> {
    let mut sub_ids = Vec::new();
    if let Some(id) = subscriber.subscription_identifier {
        sub_ids.push(id);
    }
    sub_ids
}
