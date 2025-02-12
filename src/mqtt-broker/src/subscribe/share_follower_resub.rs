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
use common_base::tools::{now_second, unique_id};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use grpc_clients::pool::ClientPool;
use log::{error, info};
use metadata_struct::mqtt::node_extend::MqttNodeExtend;
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, Login, MqttPacket, MqttProtocol, PingReq,
    PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec,
    PubRecProperties, PubRecReason, Subscribe, SubscribeProperties, SubscribeReasonCode,
    Unsubscribe, UnsubscribeProperties,
};
use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tokio::{io, select};
use tokio_util::codec::{FramedRead, FramedWrite};

use super::sub_common::{
    get_share_sub_leader, publish_message_qos0, publish_message_to_client, qos2_send_publish,
    qos2_send_pubrel, wait_packet_ack,
};
use super::subscribe_manager::SubscribeManager;
use super::subscriber::SubPublishParam;
use crate::handler::cache::{CacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;
use crate::subscribe::subscribe_manager::ShareSubShareSub;
use crate::subscribe::subscriber::Subscriber;

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
            self.start_resub_thread().await;
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn start_resub_thread(&self) {
        for (follower_resub_key, share_sub) in self.subscribe_manager.share_follower_resub.clone() {
            let metadata_cache = self.cache_manager.clone();
            let connection_manager = self.connection_manager.clone();

            match get_share_sub_leader(&self.client_pool, &share_sub.group_name).await {
                Ok(reply) => {
                    let conf = broker_mqtt_conf();
                    if conf.broker_id == reply.broker_id {
                        // remove follower sub
                        self.subscribe_manager
                            .share_follower_resub
                            .remove(&follower_resub_key);

                        // parse sub
                        let _subscribe = Subscribe {
                            packet_identifier: share_sub.packet_identifier,
                            filters: vec![share_sub.filter],
                        };
                        let _subscribe_properties = SubscribeProperties {
                            subscription_identifier: share_sub.subscription_identifier,
                            user_properties: Vec::new(),
                        };
                        // self.subscribe_manager
                        //     .add_subscribe0(
                        //         share_sub.client_id,
                        //         share_sub.protocol,
                        //         subscribe,
                        //         Some(subscribe_properties),
                        //     )
                        //     .await;
                    } else {
                        if self
                            .subscribe_manager
                            .share_follower_resub_thread
                            .contains_key(&follower_resub_key)
                        {
                            continue;
                        }

                        let (stop_sx, _) = broadcast::channel(1);
                        let subscribe_manager = self.subscribe_manager.clone();

                        // push thread
                        tokio::spawn(async move {
                            if share_sub.protocol == MqttProtocol::Mqtt4 {
                                error!(
                                    "{}",
                                    "MQTT 4 does not currently support shared subscriptions"
                                );
                            } else if share_sub.protocol == MqttProtocol::Mqtt5 {
                                let extend_info: MqttNodeExtend = match serde_json::from_str::<
                                    MqttNodeExtend,
                                >(
                                    &reply.extend_info
                                ) {
                                    Ok(da) => {
                                        subscribe_manager
                                            .share_follower_resub_thread
                                            .insert(follower_resub_key.clone(), stop_sx.clone());
                                        da
                                    }
                                    Err(e) => {
                                        error!("Failed to obtain the Leader of GroupName from the Placement Center with error message {}",e);
                                        return;
                                    }
                                };
                                match resub_sub_mqtt5(
                                    extend_info.mqtt_addr,
                                    metadata_cache,
                                    share_sub,
                                    stop_sx,
                                    connection_manager.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {}
                                    Err(e) => error!("{}", e),
                                }
                                subscribe_manager
                                    .share_follower_resub_thread
                                    .remove(&follower_resub_key);
                            }
                        });
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
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

    info!(
        "ReSub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was start successfully",
        mqtt_client_id,
        group_name,
        sub_name,
    );
    let codec = Mqtt5Codec::new();
    let socket = TcpStream::connect(leader_addr.clone()).await?;

    // split stream
    let (r_stream, w_stream) = io::split(socket);
    let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
    let write_frame_stream = FramedWrite::new(w_stream, codec.clone());

    let ws = WriteStream::new(stop_sx.clone());
    ws.add_write(write_frame_stream);
    let write_stream = Arc::new(ws);

    let follower_sub_leader_client_id = unique_id();
    let follower_sub_leader_pkid: u16 = 1;

    // Create a connection to GroupName
    let connect_pkg = build_resub_connect_pkg(
        MqttProtocol::Mqtt5.into(),
        follower_sub_leader_client_id.clone(),
    )
    .clone();
    write_stream.write_frame(connect_pkg).await;

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

                        // When a thread exits, an unsubscribed mqtt packet is sent
                        let unscribe_pkg =
                            build_resub_unsubscribe_pkg(follower_sub_leader_pkid, share_sub.clone());
                        write_stream.write_frame(unscribe_pkg).await;
                        break;
                    }
                }
            }
            val = read_frame_stream.next()=>{
                if let Some(Ok(packet)) = val{
                    let is_break = process_packet(
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
                    ).await;

                    if is_break{
                        break;
                    }else{
                        continue;
                    }
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
) -> bool {
    info!("sub follower recv:{:?}", packet);
    match packet {
        MqttPacket::ConnAck(connack, connack_properties) => {
            if connack.code == ConnectReturnCode::Success {
                // start ping thread
                start_ping_thread(write_stream.clone(), stop_sx.clone()).await;

                // When the connection is successful, a subscription request is sent
                let sub_packet =
                    build_resub_subscribe_pkg(follower_sub_leader_pkid, share_sub.clone());
                write_stream.write_frame(sub_packet).await;

                false
            } else {
                error!("client_id:[{}], group_name:[{}], sub_name:[{}] Follower forwarding subscription connection request error,
                            error message: {:?},{:?}",mqtt_client_id,group_name,sub_name,connack,connack_properties);
                true
            }
        }

        MqttPacket::SubAck(suback, _) => {
            let mut is_success = true;
            for reason in suback.return_codes.clone() {
                if !(reason
                    == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtLeastOnce)
                    || reason
                        == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::AtMostOnce)
                    || reason
                        == SubscribeReasonCode::Success(protocol::mqtt::common::QoS::ExactlyOnce)
                    || reason == SubscribeReasonCode::QoS0
                    || reason == SubscribeReasonCode::QoS1
                    || reason == SubscribeReasonCode::QoS2)
                {
                    is_success = false;
                }
            }
            !is_success
        }

        MqttPacket::Publish(mut publish, publish_properties) => {
            let cache_manager = cache_manager.clone();
            let stop_sx = stop_sx.clone();
            let connection_manager = connection_manager.clone();
            let write_stream = write_stream.clone();
            let follower_sub_leader_client_id = follower_sub_leader_client_id.to_owned();
            let mqtt_client_id = mqtt_client_id.to_owned();
            tokio::spawn(async move {
                let subscriber = Subscriber {
                    client_id: mqtt_client_id.clone(),
                    ..Default::default()
                };

                let publish_to_client_pkid: u16 = cache_manager.get_pkid(&mqtt_client_id).await;
                publish.pkid = publish_to_client_pkid;

                let sub_pub_param = SubPublishParam::new(
                    subscriber.clone(),
                    publish.clone(),
                    publish_properties,
                    0,
                    "".to_string(),
                    publish_to_client_pkid,
                );

                match publish.qos {
                    // 1. leader publish to resub thread
                    protocol::mqtt::common::QoS::AtMostOnce => {
                        publish.dup = false;
                        publish_message_qos0(
                            &cache_manager,
                            &connection_manager,
                            &sub_pub_param,
                            &stop_sx,
                        )
                        .await;
                    }

                    protocol::mqtt::common::QoS::AtLeastOnce => {
                        let (wait_puback_sx, _) = broadcast::channel(1);
                        cache_manager.add_ack_packet(
                            &mqtt_client_id,
                            publish_to_client_pkid,
                            QosAckPacketInfo {
                                sx: wait_puback_sx.clone(),
                                create_time: now_second(),
                            },
                        );

                        match resub_publish_message_qos1(
                            &cache_manager,
                            &connection_manager,
                            &sub_pub_param,
                            &stop_sx,
                            &wait_puback_sx,
                            &write_stream,
                        )
                        .await
                        {
                            Ok(()) => {
                                cache_manager
                                    .remove_pkid_info(&mqtt_client_id, publish_to_client_pkid);
                                cache_manager
                                    .remove_ack_packet(&mqtt_client_id, publish_to_client_pkid);
                            }
                            Err(e) => {
                                error!("{}", e);
                            }
                        }
                    }

                    protocol::mqtt::common::QoS::ExactlyOnce => {
                        let (wait_client_ack_sx, _) = broadcast::channel(1);

                        cache_manager.add_ack_packet(
                            &mqtt_client_id,
                            publish_to_client_pkid,
                            QosAckPacketInfo {
                                sx: wait_client_ack_sx.clone(),
                                create_time: now_second(),
                            },
                        );

                        let (wait_leader_ack_sx, _) = broadcast::channel(1);
                        cache_manager.add_ack_packet(
                            &follower_sub_leader_client_id,
                            publish.pkid,
                            QosAckPacketInfo {
                                sx: wait_leader_ack_sx.clone(),
                                create_time: now_second(),
                            },
                        );

                        match resub_publish_message_qos2(
                            &cache_manager,
                            &sub_pub_param,
                            &connection_manager,
                            &stop_sx,
                            &wait_client_ack_sx,
                            &wait_leader_ack_sx,
                            &write_stream,
                        )
                        .await
                        {
                            Ok(()) => {
                                cache_manager
                                    .remove_pkid_info(&mqtt_client_id, publish_to_client_pkid);
                                cache_manager
                                    .remove_ack_packet(&mqtt_client_id, publish_to_client_pkid);
                                cache_manager.remove_ack_packet(
                                    &follower_sub_leader_client_id,
                                    publish.pkid,
                                );
                            }
                            Err(e) => {
                                error!("{}", e);
                            }
                        }
                    }
                }
            });

            false
        }

        MqttPacket::PubRel(pubrel, _) => {
            if let Some(data) =
                cache_manager.get_ack_packet(follower_sub_leader_client_id.to_owned(), pubrel.pkid)
            {
                match data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubRel,
                    pkid: pubrel.pkid,
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            false
        }

        MqttPacket::Disconnect(_, _) => {
            info!("{}", "receive disconnect");
            true
        }

        MqttPacket::UnsubAck(_, _) => {
            // When a thread exits, an unsubscribed mqtt packet is sent
            let unscribe_pkg =
                build_resub_unsubscribe_pkg(follower_sub_leader_pkid, share_sub.clone());
            write_stream.write_frame(unscribe_pkg).await;
            true
        }
        MqttPacket::PingResp(_) => false,
        _ => {
            error!(
                "{}",
                "Rewrite subscription thread cannot recognize the currently returned package"
            );
            false
        }
    }
}

async fn start_ping_thread(write_stream: Arc<WriteStream>, stop_sx: Sender<bool>) {
    tokio::spawn(async move {
        info!("{}", "start_ping_thread start");

        loop {
            let send_ping = async {
                let ping_packet = MqttPacket::PingReq(PingReq {});
                write_stream.write_frame(ping_packet.clone()).await;
                sleep(Duration::from_secs(20)).await;
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
                _ = send_ping => {}

            }
        }
    });
}

async fn resub_publish_message_qos1(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
) -> Result<(), MqttBrokerError> {
    let mut retry_times = 0;
    let current_message_pkid = sub_pub_param.pkid;
    let mut publish = sub_pub_param.publish.clone();
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                return Ok(());
            }
        }

        let connect_id =
            if let Some(id) = metadata_cache.get_connect_id(&sub_pub_param.subscribe.client_id) {
                id
            } else {
                sleep(Duration::from_secs(1)).await;
                continue;
            };

        if let Some(conn) = metadata_cache.get_connection(connect_id) {
            if publish.payload.len() > (conn.max_packet_size as usize) {
                return Ok(());
            }
        }

        retry_times += 1;
        publish.pkid = sub_pub_param.pkid;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MqttPacket::Publish(publish.clone(), sub_pub_param.properties.clone()),
        };

        // 2. publish to mqtt client
        match publish_message_to_client(resp, sub_pub_param, connection_manager, metadata_cache)
            .await
        {
            Ok(_) => {
                // 3. wait mqtt client puback
                if let Some(data) = wait_packet_ack(wait_puback_sx).await {
                    if data.ack_type == QosAckPackageType::PubAck && data.pkid == sub_pub_param.pkid
                    {
                        // 4. puback message to sub leader
                        let puback =
                            build_resub_publish_ack(current_message_pkid, PubAckReason::Success);

                        info!("send leader:{:?}", puback);
                        write_stream.write_frame(puback.clone()).await;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to write QOS1 Publish message to response queue, failure message: {}",
                    e
                );
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// 1. send publish message
// 2. wait pubrec message
// 3. send pubrel message
// 4. wait pubcomp message
#[allow(clippy::too_many_arguments)]
pub async fn resub_publish_message_qos2(
    metadata_cache: &Arc<CacheManager>,
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
    wait_client_ack_sx: &broadcast::Sender<QosAckPackageData>,
    wait_leader_ack_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
) -> Result<(), MqttBrokerError> {
    let current_message_pkid = sub_pub_param.publish.pkid;

    // 2. Send publish message to mqtt client
    qos2_send_publish(connection_manager, metadata_cache, sub_pub_param, stop_sx).await?;

    let mut stop_rx = stop_sx.subscribe();
    loop {
        select! {
            val = stop_rx.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }
            // 3. Wait mqtt client PubRec
            val = wait_packet_ack(wait_client_ack_sx) => {
                if let Some(data) = val{

                    if data.ack_type == QosAckPackageType::PubRec && data.pkid == sub_pub_param.pkid {
                        // 4. pubrec to leader
                        let connect_id =
                            if let Some(id) = metadata_cache.get_connect_id(&sub_pub_param.subscribe.client_id) {
                                id
                            } else {
                                sleep(Duration::from_secs(1)).await;
                                continue;
                            };

                        let pubrec = build_resub_publish_rec(
                            current_message_pkid,
                            PubRecReason::Success,
                            connect_id,
                        );

                        info!("send leader:{:?}", pubrec);
                        write_stream.write_frame(pubrec).await;
                        break;
                    }
                }
            }
        }
    }

    loop {
        select! {
            val = stop_rx.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }

            // 5. Wait Sub leader PubRel
            val =  wait_packet_ack(wait_leader_ack_sx) =>{
                if let Some(data) = val{
                    if data.ack_type == QosAckPackageType::PubRel && data.pkid == current_message_pkid {
                        // 6. Send PubRel to mqtt client
                        qos2_send_pubrel(
                            metadata_cache,
                            sub_pub_param,
                            connection_manager,
                            stop_sx,
                        )
                        .await;
                        break;
                    }
                }
            }
        }
    }

    // 7. wait client pubcomp
    loop {
        select! {
            val = stop_rx.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }
            val = wait_packet_ack(wait_client_ack_sx) => {
                if let Some(data) = val{
                    if data.ack_type == QosAckPackageType::PubComp {
                        // 8.pubcomp to leader
                        let pubcomp =
                            build_resub_publish_comp(current_message_pkid, PubCompReason::Success);

                        info!("send leader:{:?}", pubcomp);
                        write_stream.write_frame(pubcomp).await;
                        break;
                    }
                }
            }
        }
    }
    Ok(())
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

fn build_resub_subscribe_pkg(
    follower_sub_leader_pkid: u16,
    share_sub: ShareSubShareSub,
) -> MqttPacket {
    let subscribe = Subscribe {
        packet_identifier: follower_sub_leader_pkid,
        filters: vec![share_sub.filter],
    };

    let subscribe_properties = SubscribeProperties {
        subscription_identifier: share_sub.subscription_identifier,
        user_properties: Vec::new(),
    };

    MqttPacket::Subscribe(subscribe, Some(subscribe_properties))
}

fn build_resub_unsubscribe_pkg(
    follower_sub_leader_pkid: u16,
    rewrite_sub: ShareSubShareSub,
) -> MqttPacket {
    MqttPacket::Unsubscribe(
        Unsubscribe {
            pkid: follower_sub_leader_pkid,
            filters: vec![rewrite_sub.filter.path],
        },
        Some(UnsubscribeProperties {
            user_properties: Vec::new(),
        }),
    )
}

fn build_resub_publish_ack(pkid: u16, reason: PubAckReason) -> MqttPacket {
    let puback = PubAck {
        pkid,
        reason: Some(reason),
    };
    let puback_properties = PubAckProperties::default();
    MqttPacket::PubAck(puback, Some(puback_properties))
}

fn build_resub_publish_rec(pkid: u16, reason: PubRecReason, mqtt_client_pkid: u64) -> MqttPacket {
    let puback = PubRec {
        pkid,
        reason: Some(reason),
    };
    let puback_properties = PubRecProperties {
        reason_string: None,
        user_properties: vec![("mqtt_client_pkid".to_string(), mqtt_client_pkid.to_string())],
    };
    MqttPacket::PubRec(puback, Some(puback_properties))
}

fn build_resub_publish_comp(pkid: u16, reason: PubCompReason) -> MqttPacket {
    let pubcomp = PubComp {
        pkid,
        reason: Some(reason),
    };
    let pubcomp_properties = PubCompProperties::default();
    MqttPacket::PubComp(pubcomp, Some(pubcomp_properties))
}

pub struct WriteStream {
    write_list:
        DashMap<String, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>>,
    key: String,
    stop_sx: Sender<bool>,
}

impl WriteStream {
    pub fn new(stop_sx: Sender<bool>) -> Self {
        WriteStream {
            key: "default".to_string(),
            write_list: DashMap::with_capacity(2),
            stop_sx,
        }
    }

    pub fn add_write(
        &self,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
    ) {
        self.write_list.insert(self.key.clone(), write);
    }

    pub async fn write_frame(&self, resp: MqttPacket) {
        loop {
            if let Ok(flag) = self.stop_sx.subscribe().try_recv() {
                if flag {
                    break;
                }
            }

            match self.write_list.try_get_mut(&self.key) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Resub Client Failed to write data to the response queue, error message: {:?}",
                                e
                            );
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    error!("Resub Client [write_frame]Connection management could not obtain an available connection.");
                }
                dashmap::try_result::TryResult::Locked => {
                    error!("Resub Client [write_frame]Connection management failed to get connection variable reference");
                }
            }
            sleep(Duration::from_secs(1)).await
        }
    }
}

#[cfg(test)]
mod tests {}
