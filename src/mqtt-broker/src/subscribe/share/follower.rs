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

use crate::common::types::ResultMqttBrokerError;
use common_base::network::{broker_not_available, is_port_open};
use common_base::tools::{get_local_ip, now_mills, now_second, unique_id};
use common_config::broker::broker_config;
use futures::StreamExt;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::node_extend::MqttNodeExtend;
use protocol::mqtt::common::{
    ConnAck, ConnAckProperties, Connect, ConnectProperties, ConnectReturnCode, Disconnect, Login,
    MqttPacket, MqttProtocol, PingReq, PubAck, PubAckProperties, PubAckReason, PubComp,
    PubCompProperties, PubCompReason, PubRec, PubRecProperties, PubRecReason, Publish,
    PublishProperties, SubAck, Subscribe, SubscribeProperties, SubscribeReasonCode, Unsubscribe,
    UnsubscribeProperties,
};
use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tokio::{io, select};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, info, warn};

use crate::common::tool::is_ignore_print;
use crate::handler::cache::{
    MQTTCacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo,
};
use crate::handler::error::MqttBrokerError;
use crate::handler::subscribe::{add_share_push_leader, ParseShareQueueSubscribeRequest};
use crate::subscribe::common::get_share_sub_leader;
use crate::subscribe::common::SubPublishParam;
use crate::subscribe::common::Subscriber;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::manager::{ShareSubShareSub, SubPushThreadData};
use crate::subscribe::push::{
    exclusive_publish_message_qos1, push_packet_to_client, qos2_send_pubrel, wait_packet_ack,
    wait_pub_rec,
};
use network_server::common::connection_manager::ConnectionManager;

use super::write::WriteStream;

#[derive(Clone)]
pub struct ProcessPacketContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub share_sub: ShareSubShareSub,
    pub stop_sx: Sender<bool>,
    pub connection_manager: Arc<ConnectionManager>,
    pub write_stream: Arc<WriteStream>,
    pub follower_sub_leader_pkid: u16,
    pub follower_sub_leader_client_id: String,
    pub mqtt_client_id: String,
    pub group_name: String,
    pub sub_name: String,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub sub_key: String,
}

#[derive(Clone)]
pub struct ConnAckContext {
    pub write_stream: Arc<WriteStream>,
    pub share_sub: ShareSubShareSub,
    pub follower_sub_leader_pkid: u16,
    pub mqtt_client_id: String,
    pub group_name: String,
    pub sub_name: String,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub sub_key: String,
    pub stop_sx: Sender<bool>,
}

#[derive(Clone)]
pub struct PublishContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub write_stream: Arc<WriteStream>,
    pub mqtt_client_id: String,
    pub follower_sub_leader_client_id: String,
    pub stop_sx: Sender<bool>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Qos2Context {
    pub metadata_cache: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub stop_sx: broadcast::Sender<bool>,
    pub wait_client_ack_sx: broadcast::Sender<QosAckPackageData>,
    pub wait_leader_ack_sx: broadcast::Sender<QosAckPackageData>,
    pub write_stream: Arc<WriteStream>,
    pub follower_sub_leader_client_id: String,
    pub mqtt_client_id: String,
    pub current_message_pkid: u16,
}

#[derive(Clone)]
pub struct ShareFollowerResub {
    pub subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<MQTTCacheManager>,
    client_pool: Arc<ClientPool>,
}

impl ShareFollowerResub {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<MQTTCacheManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        ShareFollowerResub {
            subscribe_manager,
            connection_manager,
            cache_manager,
            client_pool,
        }
    }

    pub async fn start(&self) {
        loop {
            if let Err(e) = self.start_resub_thread().await {
                error!("start_resub_thread, {}", e);
            }
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn start_resub_thread(&self) -> ResultMqttBrokerError {
        let conf = broker_config();

        for (follower_resub_key, share_sub) in self.subscribe_manager.share_follower_resub.clone() {
            let metadata_cache = self.cache_manager.clone();
            let connection_manager = self.connection_manager.clone();

            let reply = get_share_sub_leader(&self.client_pool, &share_sub.group_name).await?;

            // share follower => share leader
            if conf.broker_id == reply.broker_id {
                // remove follower sub
                self.subscribe_manager
                    .share_follower_resub
                    .remove(&follower_resub_key);

                // add share leader push
                let req = ParseShareQueueSubscribeRequest {
                    topic_name: share_sub.topic_name.to_owned(),
                    client_id: share_sub.client_id.to_owned(),
                    protocol: share_sub.protocol.clone(),
                    sub_identifier: share_sub.subscription_identifier,
                    filter: share_sub.filter.clone(),
                    pkid: share_sub.packet_identifier,
                    sub_name: share_sub.sub_name,
                    group_name: share_sub.group_name,
                };
                add_share_push_leader(&self.subscribe_manager, &req).await;
                continue;
            }

            // Add follower resub thread
            let extend_info: MqttNodeExtend =
                serde_json::from_str::<MqttNodeExtend>(&reply.extend_info)?;

            if share_sub.protocol == MqttProtocol::Mqtt3
                || share_sub.protocol == MqttProtocol::Mqtt4
            {
                continue;
            }

            if self
                .subscribe_manager
                .share_follower_resub_thread
                .contains_key(&follower_resub_key)
            {
                continue;
            }

            if !is_port_open(&extend_info.mqtt_addr) {
                warn!(
                    "Leader node {} has an unconnected network, and the Follower's subscription thread cannot start normally. client_id:[{}], group_name:[{}], sub_name:[{}]",
                    extend_info.mqtt_addr, share_sub.client_id, share_sub.group_name, share_sub.sub_name
                );
                continue;
            }

            let (stop_sx, _) = broadcast::channel(1);
            let subscribe_manager = self.subscribe_manager.clone();

            self.subscribe_manager.share_follower_resub_thread.insert(
                follower_resub_key.clone(),
                SubPushThreadData {
                    push_success_record_num: 0,
                    push_error_record_num: 0,
                    last_push_time: 0,
                    last_run_time: 0,
                    create_time: now_second(),
                    sender: stop_sx.clone(),
                },
            );

            // Follower resub thread
            tokio::spawn(async move {
                if let Err(e) = resub_sub_mqtt5(
                    extend_info.mqtt_addr,
                    metadata_cache,
                    share_sub,
                    stop_sx,
                    connection_manager.clone(),
                    subscribe_manager.clone(),
                    &follower_resub_key,
                )
                .await
                {
                    error!("resub_sub_mqtt5: {}", e);
                }
                subscribe_manager
                    .share_follower_resub_thread
                    .remove(&follower_resub_key);
            });
        }
        Ok(())
    }

    fn try_thread_gc(&self) {
        for (share_follower_key, sx) in self.subscribe_manager.share_follower_resub_thread.clone() {
            if !self
                .subscribe_manager
                .share_follower_resub
                .contains_key(&share_follower_key)
                && sx.sender.send(true).is_ok()
            {
                self.subscribe_manager
                    .share_follower_resub_thread
                    .remove(&share_follower_key);
            }
        }
    }
}

async fn resub_sub_mqtt5(
    leader_addr: String,
    cache_manager: Arc<MQTTCacheManager>,
    share_sub: ShareSubShareSub,
    stop_sx: Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
    follower_resub_key: &str,
) -> ResultMqttBrokerError {
    let mqtt_client_id = share_sub.client_id.clone();
    let group_name = share_sub.group_name.clone();
    let sub_name = share_sub.sub_name.clone();
    let follower_sub_leader_client_id = format!("resub_{}_{}", get_local_ip(), unique_id());
    let follower_sub_leader_pkid: u16 = 1;

    info!(
        "ReSub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was start successfully",
        mqtt_client_id,
        group_name,
        sub_name,
    );

    // Connect to the share subscribe leader
    let (mut read_frame_stream, write_stream) =
        connection_to_leader(&leader_addr, &follower_sub_leader_client_id, &stop_sx).await?;

    // Continuously receive back packet information from the Server
    let mut stop_rx: broadcast::Receiver<bool> = stop_sx.subscribe();
    loop {
        select! {
            val = stop_rx.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        info!(
                            "Rewrite sub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was stopped successfully",
                            mqtt_client_id,
                            group_name,
                            sub_name,
                        );
                        try_close_connection(&write_stream, &share_sub.filter.path, follower_sub_leader_pkid).await;
                        break;
                    }
                }
            }

            val = read_frame_stream.next()=>{
                if let Some(Ok(packet)) = val{
                    let context = ProcessPacketContext {
                        cache_manager: cache_manager.clone(),
                        share_sub: share_sub.clone(),
                        stop_sx: stop_sx.clone(),
                        connection_manager: connection_manager.clone(),
                        write_stream: write_stream.clone(),
                        follower_sub_leader_pkid,
                        follower_sub_leader_client_id: follower_sub_leader_client_id.clone(),
                        mqtt_client_id: mqtt_client_id.clone(),
                        group_name: group_name.clone(),
                        sub_name: sub_name.clone(),
                        subscribe_manager: subscribe_manager.clone(),
                        sub_key: follower_resub_key.to_string(),
                    };
                    if let Err(e) = process_packet(&context, packet).await {
                        error!("Share follower node failed to process the package returned by the Leader. Error message: {}, key: {} ", e, follower_resub_key);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn process_packet(
    context: &ProcessPacketContext,
    packet: MqttPacket,
) -> ResultMqttBrokerError {
    if !is_ignore_print(&packet) {
        info!("Follower node receives packet:{:?}", packet);
    }

    match packet {
        MqttPacket::ConnAck(connack, connack_properties) => {
            let conn_ack_context = ConnAckContext {
                write_stream: context.write_stream.clone(),
                share_sub: context.share_sub.clone(),
                follower_sub_leader_pkid: context.follower_sub_leader_pkid,
                mqtt_client_id: context.mqtt_client_id.clone(),
                group_name: context.group_name.clone(),
                sub_name: context.sub_name.clone(),
                subscribe_manager: context.subscribe_manager.clone(),
                sub_key: context.sub_key.clone(),
                stop_sx: context.stop_sx.clone(),
            };
            process_conn_ack_packet(&connack, &connack_properties, &conn_ack_context).await?;
        }

        MqttPacket::SubAck(sub_ack, _) => {
            process_sub_ack(sub_ack).await?;
        }

        MqttPacket::Publish(publish, publish_properties) => {
            let raw_cache_manager = context.cache_manager.clone();
            let raw_connection_manager = context.connection_manager.clone();
            let raw_write_stream = context.write_stream.clone();
            let raw_stop_sx = context.stop_sx.clone();
            let raw_follower_sub_leader_client_id =
                context.follower_sub_leader_client_id.to_string();
            let raw_mqtt_client_id = context.mqtt_client_id.to_string();
            tokio::spawn(async move {
                let publish_context = PublishContext {
                    cache_manager: raw_cache_manager,
                    connection_manager: raw_connection_manager,
                    write_stream: raw_write_stream,
                    mqtt_client_id: raw_mqtt_client_id,
                    follower_sub_leader_client_id: raw_follower_sub_leader_client_id,
                    stop_sx: raw_stop_sx,
                };
                if let Err(e) =
                    process_publish_packet(&publish_context, publish, publish_properties).await
                {
                    error!(
                        "Follower node failed to process the Publish packet, error message: {}",
                        e
                    );
                }
            });
        }

        MqttPacket::PubRel(pubrel, _) => {
            if let Some(data) = context
                .cache_manager
                .pkid_metadata
                .get_ack_packet(&context.follower_sub_leader_client_id, pubrel.pkid)
            {
                if let Err(e) = data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubRel,
                    pkid: pubrel.pkid,
                }) {
                    return Err(MqttBrokerError::CommonError(e.to_string()));
                }
            }
        }

        MqttPacket::Disconnect(_, _) => {
            info!("{}", "receive disconnect");
        }

        MqttPacket::UnsubAck(_, _) => {
            un_subscribe_to_leader(
                context.follower_sub_leader_pkid,
                &context.write_stream,
                &context.share_sub.filter.path,
            )
            .await?;
        }

        MqttPacket::PingResp(_) => {}

        _ => {
            error!(
                "{}",
                "Rewrite subscription thread cannot recognize the currently returned package"
            );
        }
    }
    Ok(())
}

async fn process_conn_ack_packet(
    connack: &ConnAck,
    connack_properties: &Option<ConnAckProperties>,
    context: &ConnAckContext,
) -> ResultMqttBrokerError {
    if connack.code == ConnectReturnCode::Success {
        // Ping
        start_ping_thread(
            context.subscribe_manager.to_owned(),
            context.write_stream.clone(),
            context.sub_key.to_owned(),
            context.stop_sx.clone(),
        )
        .await;

        // Subscribe
        subscribe_to_leader(
            context.follower_sub_leader_pkid,
            &context.share_sub,
            &context.write_stream,
        )
        .await?;
        return Ok(());
    }
    Err(MqttBrokerError::CommonError(format!("client_id:[{}], group_name:[{}], sub_name:[{}] Follower forwarding subscription connection request error,
                            error message: {connack:?},{connack_properties:?}", context.mqtt_client_id, context.group_name, context.sub_name)))
}

async fn process_sub_ack(suback: SubAck) -> ResultMqttBrokerError {
    for reason in suback.return_codes.clone() {
        if !(reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtLeastOnce)
            || reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtMostOnce)
            || reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::ExactlyOnce)
            || reason == SubscribeReasonCode::QoS0
            || reason == SubscribeReasonCode::QoS1
            || reason == SubscribeReasonCode::QoS2)
        {
            return Err(MqttBrokerError::CommonError(format!("{suback:?}")));
        }
    }
    Ok(())
}

async fn process_publish_packet(
    context: &PublishContext,
    mut publish: Publish,
    publish_properties: Option<PublishProperties>,
) -> ResultMqttBrokerError {
    let subscriber = Subscriber {
        client_id: context.mqtt_client_id.clone(),
        ..Default::default()
    };

    let publish_to_client_pkid = context
        .cache_manager
        .pkid_metadata
        .generate_pkid(&context.mqtt_client_id, &publish.qos)
        .await;
    publish.p_kid = publish_to_client_pkid;

    let packet = MqttPacket::Publish(publish.clone(), publish_properties);
    let sub_pub_param = SubPublishParam::new(
        subscriber.clone(),
        packet,
        now_mills(),
        "".to_string(),
        publish_to_client_pkid,
    );

    match publish.qos {
        protocol::mqtt::common::QoS::AtMostOnce => {
            push_packet_to_client(
                &context.cache_manager,
                &context.connection_manager,
                &sub_pub_param,
                &context.stop_sx,
            )
            .await?;
        }

        protocol::mqtt::common::QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            context.cache_manager.pkid_metadata.add_ack_packet(
                &context.mqtt_client_id,
                publish_to_client_pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_mills(),
                },
            );

            resub_publish_message_qos1(
                &context.cache_manager,
                &context.connection_manager,
                &sub_pub_param,
                &context.stop_sx,
                &wait_puback_sx,
                &context.write_stream,
            )
            .await?;

            context
                .cache_manager
                .pkid_metadata
                .remove_ack_packet(&context.mqtt_client_id, publish_to_client_pkid);
        }

        protocol::mqtt::common::QoS::ExactlyOnce => {
            let (wait_client_ack_sx, _) = broadcast::channel(1);

            context.cache_manager.pkid_metadata.add_ack_packet(
                &context.mqtt_client_id,
                publish_to_client_pkid,
                QosAckPacketInfo {
                    sx: wait_client_ack_sx.clone(),
                    create_time: now_mills(),
                },
            );

            let (wait_leader_ack_sx, _) = broadcast::channel(1);
            context.cache_manager.pkid_metadata.add_ack_packet(
                &context.follower_sub_leader_client_id,
                publish.p_kid,
                QosAckPacketInfo {
                    sx: wait_leader_ack_sx.clone(),
                    create_time: now_mills(),
                },
            );

            resub_publish_message_qos2(
                context,
                &sub_pub_param,
                &wait_client_ack_sx,
                &wait_leader_ack_sx,
                publish.p_kid,
            )
            .await?;

            context
                .cache_manager
                .pkid_metadata
                .remove_ack_packet(&context.mqtt_client_id, publish_to_client_pkid);
            context
                .cache_manager
                .pkid_metadata
                .remove_ack_packet(&context.follower_sub_leader_client_id, publish.p_kid);
        }
    }
    Ok(())
}

async fn start_ping_thread(
    subscribe_manager: Arc<SubscribeManager>,
    write_stream: Arc<WriteStream>,
    sub_key: String,
    stop_sx: Sender<bool>,
) {
    tokio::spawn(async move {
        loop {
            let send_ping = async || -> ResultMqttBrokerError {
                let ping_packet = MqttPacket::PingReq(PingReq {});
                if let Err(e) = write_stream.write_frame(ping_packet.clone()).await {
                    if broker_not_available(&e.to_string()) && !is_port_open(&write_stream.address)
                    {
                        info!("Heartbeat detection: Leader node {} has no network connection. Exiting subscription thread.sub_key:{}", write_stream.address, sub_key);
                        if let Some(sx) =
                            subscribe_manager.share_follower_resub_thread.get(&sub_key)
                        {
                            if let Err(se) = sx.sender.send(true) {
                                error!(
                                    "Follower Resub thread failed to stop. Error message: {}",
                                    se
                                );
                            }
                        }
                        return Err(e);
                    } else {
                        error!("Failed to send PingReq packet: {}", e);
                    }
                }
                Ok(())
            };

            let mut stop_rx = stop_sx.subscribe();

            select! {
                val = stop_rx.recv() => {
                    if let Ok(flag) = val {
                        if flag {
                            break;
                        }
                    }
                },
                res = send_ping() => {
                    if  res.is_err() {
                        break;
                    }
                    sleep(Duration::from_secs(1)).await;
                }

            }
        }
    });
}

// 1. Leader Publish to Follower
// 2. Follower Publish to Client
// 3. Follower receive cli puback
// 4. Follower send puback to Leader
async fn resub_publish_message_qos1(
    cache_manager: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
) -> ResultMqttBrokerError {
    exclusive_publish_message_qos1(
        cache_manager,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_puback_sx,
    )
    .await?;

    let current_message_pkid = sub_pub_param.pkid;
    let puback = PubAck {
        pkid: current_message_pkid,
        reason: Some(PubAckReason::Success),
    };
    let puback_properties = PubAckProperties::default();
    let puback = MqttPacket::PubAck(puback, Some(puback_properties));

    write_stream.write_frame(puback.clone()).await?;
    Ok(())
}

// 1. Leader Publish to Follower
// 2. Follower Publish to Client
// 3. Follower receive cli pubrec
// 4. Follower send pubrec to Leader
// 5. Follower wait Leader pubrel
// 6. Follower send pubrel to Client
// 7. Follower wait Client pubcomp
// 8. Follower send pubcomp to Leader
pub async fn resub_publish_message_qos2(
    context: &PublishContext,
    sub_pub_param: &SubPublishParam,
    wait_client_ack_sx: &broadcast::Sender<QosAckPackageData>,
    wait_leader_ack_sx: &broadcast::Sender<QosAckPackageData>,
    current_message_pkid: u16,
) -> ResultMqttBrokerError {
    debug!("{}", sub_pub_param.group_id);

    // 1. Publish message to client
    push_packet_to_client(
        &context.cache_manager,
        &context.connection_manager,
        sub_pub_param,
        &context.stop_sx,
    )
    .await?;

    // 2. Wait client pubrec
    wait_pub_rec(
        &context.cache_manager,
        &context.connection_manager,
        sub_pub_param,
        &context.stop_sx,
        wait_client_ack_sx,
    )
    .await?;

    // 3. Send pubrec to leader
    publish_rec_to_leader(&context.write_stream, current_message_pkid).await?;

    // 4. Wait leader pubrel
    wait_packet_ack(
        wait_leader_ack_sx,
        "Pubrel",
        &context.follower_sub_leader_client_id,
    )
    .await?;

    // 5. Send pubrel to client
    qos2_send_pubrel(
        &context.cache_manager,
        sub_pub_param,
        &context.connection_manager,
        &context.stop_sx,
    )
    .await?;

    // 6. Wait client pubcomp
    wait_packet_ack(wait_client_ack_sx, "PubComp", &context.mqtt_client_id).await?;

    // 7. Send pubcomp to leader
    publish_comp_to_leader(
        &context.write_stream,
        current_message_pkid,
        PubCompReason::Success,
    )
    .await?;

    Ok(())
}

async fn try_close_connection(
    write_stream: &Arc<WriteStream>,
    path: &str,
    follower_sub_leader_pkid: u16,
) {
    let _ = un_subscribe_to_leader(follower_sub_leader_pkid, write_stream, path).await;
    let _ = disconnect_to_leader(write_stream).await;
}

async fn disconnect_to_leader(write_stream: &Arc<WriteStream>) -> ResultMqttBrokerError {
    let packet = MqttPacket::Disconnect(
        Disconnect {
            reason_code: Some(protocol::mqtt::common::DisconnectReasonCode::NormalDisconnection),
        },
        None,
    );
    write_stream.write_frame(packet.clone()).await?;
    Ok(())
}

async fn connection_to_leader(
    leader_addr: &str,
    follower_sub_leader_client_id: &str,
    stop_sx: &Sender<bool>,
) -> Result<
    (
        FramedRead<tokio::io::ReadHalf<tokio::net::TcpStream>, Mqtt5Codec>,
        Arc<WriteStream>,
    ),
    MqttBrokerError,
> {
    let codec = Mqtt5Codec::new();
    let socket = TcpStream::connect(leader_addr).await?;
    // split stream
    let (r_stream, w_stream) = io::split(socket);
    let read_frame_stream = FramedRead::new(r_stream, codec.clone());
    let write_frame_stream = FramedWrite::new(w_stream, codec.clone());

    let ws = WriteStream::new(leader_addr.to_string(), stop_sx.clone());
    ws.add_write(write_frame_stream);
    let write_stream = Arc::new(ws);

    // Create a connection to leader
    let connect_pkg = build_resub_connect_pkg(
        MqttProtocol::Mqtt5.into(),
        follower_sub_leader_client_id.to_string(),
    );
    write_stream.write_frame(connect_pkg).await?;
    Ok((read_frame_stream, write_stream))
}

async fn publish_rec_to_leader(
    write_stream: &Arc<WriteStream>,
    current_message_pkid: u16,
) -> ResultMqttBrokerError {
    let puback = PubRec {
        pkid: current_message_pkid,
        reason: Some(PubRecReason::Success),
    };
    let puback_properties = PubRecProperties {
        reason_string: None,
        user_properties: vec![("follower".to_string(), "true".to_string())],
    };
    let pubrec = MqttPacket::PubRec(puback, Some(puback_properties));
    write_stream.write_frame(pubrec.clone()).await?;
    info!("Follower node sent PubRec to leader, pubrec: {:?}", pubrec);
    Ok(())
}

async fn publish_comp_to_leader(
    write_stream: &Arc<WriteStream>,
    pkid: u16,
    reason: PubCompReason,
) -> ResultMqttBrokerError {
    let pubcomp = PubComp {
        pkid,
        reason: Some(reason),
    };
    let pubcomp_properties = PubCompProperties::default();
    let pubcomp = MqttPacket::PubComp(pubcomp, Some(pubcomp_properties));
    write_stream.write_frame(pubcomp).await
}

async fn subscribe_to_leader(
    follower_sub_leader_pkid: u16,
    share_sub: &ShareSubShareSub,
    write_stream: &Arc<WriteStream>,
) -> ResultMqttBrokerError {
    let subscribe = Subscribe {
        packet_identifier: follower_sub_leader_pkid,
        filters: vec![share_sub.filter.clone()],
    };

    let subscribe_properties = SubscribeProperties {
        subscription_identifier: share_sub.subscription_identifier,
        user_properties: Vec::new(),
    };

    let pkg = MqttPacket::Subscribe(subscribe, Some(subscribe_properties));
    write_stream.write_frame(pkg).await
}

async fn un_subscribe_to_leader(
    follower_sub_leader_pkid: u16,
    write_stream: &Arc<WriteStream>,
    path: &str,
) -> ResultMqttBrokerError {
    let pkg = MqttPacket::Unsubscribe(
        Unsubscribe {
            pkid: follower_sub_leader_pkid,
            filters: vec![path.to_string()],
        },
        Some(UnsubscribeProperties {
            user_properties: Vec::new(),
        }),
    );
    write_stream.write_frame(pkg).await
}

fn build_resub_connect_pkg(protocol_level: u8, client_id: String) -> MqttPacket {
    let conf = broker_config();
    let connect = Connect {
        keep_alive: 6000,
        client_id,
        clean_session: true,
    };

    let properties = ConnectProperties {
        session_expiry_interval: Some(60),
        user_properties: Vec::new(),
        ..Default::default()
    };

    let login = Login {
        username: conf.mqtt_runtime.default_user.clone(),
        password: conf.mqtt_runtime.default_password.clone(),
    };

    MqttPacket::Connect(
        protocol_level,
        connect,
        Some(properties),
        None,
        None,
        Some(login),
    )
}

#[cfg(test)]
mod tests {}
