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

use std::sync::Arc;
use std::time::Duration;

use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::tools::{now_mills, unique_id};
use futures::StreamExt;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::node_extend::MqttNodeExtend;
use protocol::mqtt::common::{
    ConnAck, ConnAckProperties, Connect, ConnectProperties, ConnectReturnCode, Login, MqttPacket,
    MqttProtocol, PingReq, PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties,
    PubCompReason, PubRec, PubRecProperties, PubRecReason, Publish, PublishProperties, SubAck,
    Subscribe, SubscribeProperties, SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
};
use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tokio::{io, select};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, info};

use crate::handler::cache::{CacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::subscribe::{add_share_push_leader, ParseShareQueueSubscribeRequest};
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::common::SubPublishParam;
use crate::subscribe::common::Subscriber;
use crate::subscribe::common::{get_pkid, get_share_sub_leader};
use crate::subscribe::manager::ShareSubShareSub;
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::push::{
    exclusive_publish_message_qos1, push_packet_to_client, qos2_send_pubrel, wait_packet_ack,
    wait_pub_rec,
};

use super::write::WriteStream;

#[derive(Clone)]
pub struct ShareFollowerResub {
    pub subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
}

impl ShareFollowerResub {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
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
                error!("{}", e);
            }
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn start_resub_thread(&self) -> Result<(), MqttBrokerError> {
        let conf = broker_mqtt_conf();

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
                    topic_id: share_sub.topic_id.to_owned(),
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
            let extend_info = serde_json::from_str::<MqttNodeExtend>(&reply.extend_info)?;

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

            let (stop_sx, _) = broadcast::channel(1);
            let subscribe_manager = self.subscribe_manager.clone();

            self.subscribe_manager
                .share_follower_resub_thread
                .insert(follower_resub_key.clone(), stop_sx.clone());

            // Follower resub thread
            tokio::spawn(async move {
                if let Err(e) = resub_sub_mqtt5(
                    extend_info.mqtt_addr,
                    metadata_cache,
                    share_sub,
                    stop_sx,
                    connection_manager.clone(),
                )
                .await
                {
                    error!("{}", e);
                }
                subscribe_manager
                    .share_follower_resub_thread
                    .remove(&follower_resub_key);
            });
        }
        Ok(())
    }

    fn try_thread_gc(&self) {
        for (share_fllower_key, sx) in self.subscribe_manager.share_follower_resub_thread.clone() {
            if !self
                .subscribe_manager
                .share_follower_resub
                .contains_key(&share_fllower_key)
                && sx.send(true).is_ok()
            {
                self.subscribe_manager
                    .share_follower_resub
                    .remove(&share_fllower_key);
            }
        }
    }
}

async fn resub_sub_mqtt5(
    leader_addr: String,
    cache_manager: Arc<CacheManager>,
    share_sub: ShareSubShareSub,
    stop_sx: Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
) -> Result<(), MqttBrokerError> {
    let mqtt_client_id = share_sub.client_id.clone();
    let group_name = share_sub.group_name.clone();
    let sub_name = share_sub.sub_name.clone();
    let follower_sub_leader_client_id = unique_id();
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
                        debug!(
                            "Rewrite sub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was stopped successfully",
                            mqtt_client_id,
                            group_name,
                            sub_name,
                        );
                        un_subscribe_to_leader(follower_sub_leader_pkid, &write_stream, &share_sub.filter.path).await;
                        break;
                    }
                }
            }

            val = read_frame_stream.next()=>{
                if let Some(Ok(packet)) = val{
                    if let Err(e) =  process_packet(
                        &cache_manager,
                        &share_sub,
                        &stop_sx,
                        &connection_manager,
                        packet,
                        &write_stream,
                        follower_sub_leader_pkid,
                        &follower_sub_leader_client_id,
                        &mqtt_client_id,
                        &group_name,
                        &sub_name,
                    ).await{
                        error!("{}", e);
                        continue;
                    }
                    break;
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_packet(
    cache_manager: &Arc<CacheManager>,
    share_sub: &ShareSubShareSub,
    stop_sx: &Sender<bool>,
    connection_manager: &Arc<ConnectionManager>,
    packet: MqttPacket,
    write_stream: &Arc<WriteStream>,
    follower_sub_leader_pkid: u16,
    follower_sub_leader_client_id: &str,
    mqtt_client_id: &str,
    group_name: &str,
    sub_name: &str,
) -> Result<(), MqttBrokerError> {
    match packet {
        MqttPacket::ConnAck(connack, connack_properties) => {
            process_conn_ack_packet(
                &connack,
                &connack_properties,
                write_stream,
                share_sub,
                follower_sub_leader_pkid,
                mqtt_client_id,
                group_name,
                sub_name,
                stop_sx,
            )
            .await?;
        }

        MqttPacket::SubAck(suback, _) => {
            process_sub_ack(suback).await?;
        }

        MqttPacket::Publish(publish, publish_properties) => {
            process_publish_packet(
                cache_manager,
                connection_manager,
                write_stream,
                publish,
                publish_properties,
                mqtt_client_id,
                follower_sub_leader_client_id,
                stop_sx,
            )
            .await?;
        }

        MqttPacket::PubRel(pubrel, _) => {
            if let Some(data) =
                cache_manager.get_ack_packet(follower_sub_leader_client_id, pubrel.pkid)
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
                follower_sub_leader_pkid,
                write_stream,
                &share_sub.filter.path,
            )
            .await;
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

#[allow(clippy::too_many_arguments)]
async fn process_conn_ack_packet(
    connack: &ConnAck,
    connack_properties: &Option<ConnAckProperties>,
    write_stream: &Arc<WriteStream>,
    share_sub: &ShareSubShareSub,
    follower_sub_leader_pkid: u16,
    mqtt_client_id: &str,
    group_name: &str,
    sub_name: &str,
    stop_sx: &Sender<bool>,
) -> Result<(), MqttBrokerError> {
    if connack.code == ConnectReturnCode::Success {
        // Ping
        start_ping_thread(write_stream.clone(), stop_sx.clone()).await;

        // Subscribe
        subscribe_to_leader(follower_sub_leader_pkid, share_sub, write_stream).await;
    }
    Err(MqttBrokerError::CommonError(format!("client_id:[{}], group_name:[{}], sub_name:[{}] Follower forwarding subscription connection request error,
                            error message: {:?},{:?}",mqtt_client_id,group_name,sub_name,connack,connack_properties)))
}

async fn process_sub_ack(suback: SubAck) -> Result<(), MqttBrokerError> {
    for reason in suback.return_codes.clone() {
        if !(reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtLeastOnce)
            || reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtMostOnce)
            || reason == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::ExactlyOnce)
            || reason == SubscribeReasonCode::QoS0
            || reason == SubscribeReasonCode::QoS1
            || reason == SubscribeReasonCode::QoS2)
        {
            return Err(MqttBrokerError::CommonError(format!("{:?}", suback)));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_publish_packet(
    cache_manager: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    write_stream: &Arc<WriteStream>,
    mut publish: Publish,
    publish_properties: Option<PublishProperties>,
    mqtt_client_id: &str,
    follower_sub_leader_client_id: &str,
    stop_sx: &Sender<bool>,
) -> Result<(), MqttBrokerError> {
    let subscriber = Subscriber {
        client_id: mqtt_client_id.to_string(),
        ..Default::default()
    };

    let publish_to_client_pkid = get_pkid(cache_manager, mqtt_client_id).await;
    publish.pkid = publish_to_client_pkid;

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
            push_packet_to_client(cache_manager, connection_manager, &sub_pub_param, stop_sx)
                .await?;
        }

        protocol::mqtt::common::QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            cache_manager.add_ack_packet(
                mqtt_client_id,
                publish_to_client_pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_mills(),
                },
            );

            resub_publish_message_qos1(
                cache_manager,
                connection_manager,
                &sub_pub_param,
                stop_sx,
                &wait_puback_sx,
                write_stream,
            )
            .await?;

            cache_manager.remove_ack_packet(mqtt_client_id, publish_to_client_pkid);
        }

        protocol::mqtt::common::QoS::ExactlyOnce => {
            let (wait_client_ack_sx, _) = broadcast::channel(1);

            cache_manager.add_ack_packet(
                mqtt_client_id,
                publish_to_client_pkid,
                QosAckPacketInfo {
                    sx: wait_client_ack_sx.clone(),
                    create_time: now_mills(),
                },
            );

            let (wait_leader_ack_sx, _) = broadcast::channel(1);
            cache_manager.add_ack_packet(
                follower_sub_leader_client_id,
                publish.pkid,
                QosAckPacketInfo {
                    sx: wait_leader_ack_sx.clone(),
                    create_time: now_mills(),
                },
            );

            resub_publish_message_qos2(
                cache_manager,
                &sub_pub_param,
                connection_manager,
                stop_sx,
                &wait_client_ack_sx,
                &wait_leader_ack_sx,
                write_stream,
                follower_sub_leader_client_id,
                mqtt_client_id,
                publish.pkid,
            )
            .await?;

            cache_manager.remove_ack_packet(mqtt_client_id, publish_to_client_pkid);
            cache_manager.remove_ack_packet(follower_sub_leader_client_id, publish.pkid);
        }
    }
    Ok(())
}

async fn start_ping_thread(write_stream: Arc<WriteStream>, stop_sx: Sender<bool>) {
    tokio::spawn(async move {
        loop {
            let send_ping = async {
                let ping_packet = MqttPacket::PingReq(PingReq {});
                write_stream.write_frame(ping_packet.clone()).await;
            };

            let mut stop_rx = stop_sx.subscribe();
            select! {
                val = stop_rx.recv() => {
                    if let Ok(flag) = val {
                        if flag {
                            info!("{}","start_ping_thread stop");
                            break;
                        }
                    }
                },
                _ = send_ping => {
                    sleep(Duration::from_secs(1)).await;
                }

            }
        }
    });
}

async fn resub_publish_message_qos1(
    cache_manager: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
) -> Result<(), MqttBrokerError> {
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

    write_stream.write_frame(puback.clone()).await;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn resub_publish_message_qos2(
    metadata_cache: &Arc<CacheManager>,
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
    wait_client_ack_sx: &broadcast::Sender<QosAckPackageData>,
    wait_leader_ack_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
    follower_sub_leader_client_id: &str,
    mqtt_client_id: &str,
    current_message_pkid: u16,
) -> Result<(), MqttBrokerError> {
    debug!("{}", sub_pub_param.group_id);

    // 1. Publish message to client
    push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

    // 2. Wait client pubrec
    wait_pub_rec(
        metadata_cache,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_client_ack_sx,
    )
    .await?;

    // 3. Send pubrec to leader
    publish_rec_to_leader(write_stream, current_message_pkid).await;

    // 4. Wait leader pubrel
    wait_packet_ack(wait_leader_ack_sx, "Pubrel", follower_sub_leader_client_id).await?;

    // 5. Send pubrel to client
    qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx).await?;

    // 6. Wait client pubcomp
    wait_packet_ack(wait_client_ack_sx, "PubComp", mqtt_client_id).await?;

    // 7. Send pubcomp to leader
    publish_comp_to_leader(write_stream, current_message_pkid, PubCompReason::Success).await;

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

    let ws = WriteStream::new(stop_sx.clone());
    ws.add_write(write_frame_stream);
    let write_stream = Arc::new(ws);

    // Create a connection to leader
    let connect_pkg = build_resub_connect_pkg(
        MqttProtocol::Mqtt5.into(),
        follower_sub_leader_client_id.to_string(),
    );
    write_stream.write_frame(connect_pkg).await;
    Ok((read_frame_stream, write_stream))
}

async fn publish_rec_to_leader(write_stream: &Arc<WriteStream>, current_message_pkid: u16) {
    let puback = PubRec {
        pkid: current_message_pkid,
        reason: Some(PubRecReason::Success),
    };
    let puback_properties = PubRecProperties::default();
    let pubrec = MqttPacket::PubRec(puback, Some(puback_properties));
    write_stream.write_frame(pubrec).await;
}

async fn publish_comp_to_leader(write_stream: &Arc<WriteStream>, pkid: u16, reason: PubCompReason) {
    let pubcomp = PubComp {
        pkid,
        reason: Some(reason),
    };
    let pubcomp_properties = PubCompProperties::default();
    let pubcomp = MqttPacket::PubComp(pubcomp, Some(pubcomp_properties));
    write_stream.write_frame(pubcomp).await;
}

async fn subscribe_to_leader(
    follower_sub_leader_pkid: u16,
    share_sub: &ShareSubShareSub,
    write_stream: &Arc<WriteStream>,
) {
    let subscribe = Subscribe {
        packet_identifier: follower_sub_leader_pkid,
        filters: vec![share_sub.filter.clone()],
    };

    let subscribe_properties = SubscribeProperties {
        subscription_identifier: share_sub.subscription_identifier,
        user_properties: Vec::new(),
    };

    let pkg = MqttPacket::Subscribe(subscribe, Some(subscribe_properties));
    write_stream.write_frame(pkg).await;
}

async fn un_subscribe_to_leader(
    follower_sub_leader_pkid: u16,
    write_stream: &Arc<WriteStream>,
    path: &str,
) {
    let pkg = MqttPacket::Unsubscribe(
        Unsubscribe {
            pkid: follower_sub_leader_pkid,
            filters: vec![path.to_string()],
        },
        Some(UnsubscribeProperties {
            user_properties: Vec::new(),
        }),
    );
    write_stream.write_frame(pkg).await;
}

fn build_resub_connect_pkg(protocol_level: u8, client_id: String) -> MqttPacket {
    let conf = broker_mqtt_conf();
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
        username: conf.system.default_user.clone(),
        password: conf.system.default_password.clone(),
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
