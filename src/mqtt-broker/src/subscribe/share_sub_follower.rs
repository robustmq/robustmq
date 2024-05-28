use super::{
    exclusive_sub::publish_message_qos0,
    sub_manager::SubscribeManager,
    subscribe::{
        get_share_sub_leader, publish_to_response_queue, share_sub_rewrite_publish_flag,
        wait_packet_ack,
    },
};
use crate::{
    core::metadata_cache::MetadataCacheManager,
    qos::ack_manager::{AckManager, AckPackageData, AckPackageType, AckPacketInfo},
    server::{tcp::packet::ResponsePackage, MQTTProtocol},
    subscribe::sub_manager::ShareSubShareSub,
};
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    errors::RobustMQError,
    log::{error, info},
    tools::{now_second, unique_id},
};
use futures::{SinkExt, StreamExt};
use protocol::{
    mqtt::{
        Connect, ConnectProperties, ConnectReturnCode, Login, MQTTPacket, PubAck, PubAckProperties,
        PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec, PubRecProperties,
        PubRecReason, PubRel, Publish, PublishProperties, Subscribe, SubscribeProperties,
        SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
    },
    mqttv5::codec::Mqtt5Codec,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    io,
    net::TcpStream,
    sync::broadcast::{self, Sender},
    time::sleep,
};
use tokio_util::codec::{FramedRead, FramedWrite};

#[derive(Clone)]
pub struct SubscribeShareFollower {
    pub subscribe_manager: Arc<SubscribeManager>,
    pub ack_manager: Arc<AckManager>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
    metadata_cache: Arc<MetadataCacheManager>,
    client_poll: Arc<ClientPool>,
}

impl SubscribeShareFollower {
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        response_queue_sx4: broadcast::Sender<ResponsePackage>,
        response_queue_sx5: broadcast::Sender<ResponsePackage>,
        metadata_cache: Arc<MetadataCacheManager>,
        client_poll: Arc<ClientPool>,
        ack_manager: Arc<AckManager>,
    ) -> Self {
        return SubscribeShareFollower {
            subscribe_manager,
            response_queue_sx4,
            response_queue_sx5,
            metadata_cache,
            client_poll,
            ack_manager,
        };
    }

    pub async fn start(&self) {
        loop {
            self.start_resub_thread().await;
            self.try_thread_gc();
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn start_resub_thread(&self) {
        for (follower_resub_key, share_sub) in
            self.subscribe_manager.share_follower_subscribe.clone()
        {
            let metadata_cache = self.metadata_cache.clone();
            let response_queue_sx4 = self.response_queue_sx4.clone();
            let response_queue_sx5 = self.response_queue_sx5.clone();
            let ack_manager = self.ack_manager.clone();

            match get_share_sub_leader(
                self.client_poll.clone(),
                share_sub.group_name.clone(),
                share_sub.sub_name.clone(),
            )
            .await
            {
                Ok(reply) => {
                    let conf = broker_mqtt_conf();
                    if conf.broker_id == reply.broker_id {
                        // remove follower sub
                        self.subscribe_manager
                            .share_follower_subscribe
                            .remove(&follower_resub_key);

                        // parse sub
                        let subscribe = Subscribe {
                            packet_identifier: share_sub.packet_identifier,
                            filters: vec![share_sub.filter],
                        };
                        let subscribe_poperties = SubscribeProperties {
                            subscription_identifier: share_sub.subscription_identifier,
                            user_properties: Vec::new(),
                        };
                        self.subscribe_manager
                            .add_subscribe(
                                share_sub.client_id,
                                share_sub.protocol,
                                subscribe,
                                Some(subscribe_poperties),
                            )
                            .await;
                    } else {
                        let (stop_sx, _) = broadcast::channel(1);
                        self.subscribe_manager
                            .share_follower_resub_thread
                            .insert(follower_resub_key, stop_sx.clone());

                        tokio::spawn(async move {
                            if share_sub.protocol == MQTTProtocol::MQTT4 {
                                error(
                                    "MQTT 4 does not currently support shared subscriptions"
                                        .to_string(),
                                );
                            } else if share_sub.protocol == MQTTProtocol::MQTT5 {
                                resub_sub_mqtt5(
                                    ack_manager,
                                    reply.broker_ip,
                                    metadata_cache,
                                    share_sub,
                                    stop_sx,
                                    response_queue_sx4,
                                    response_queue_sx5,
                                )
                                .await;
                            }
                        });
                    }
                }
                Err(e) => error(e.to_string()),
            }
        }
    }

    fn try_thread_gc(&self) {
        for (share_fllower_key, sx) in self.subscribe_manager.share_follower_resub_thread.clone() {
            if !self
                .subscribe_manager
                .share_follower_subscribe
                .contains_key(&share_fllower_key)
            {
                match sx.send(true) {
                    Ok(_) => {
                        self.subscribe_manager
                            .share_follower_subscribe
                            .remove(&share_fllower_key);
                    }
                    Err(_) => {}
                }
            }
        }
    }
}

async fn resub_sub_mqtt5(
    ack_manager: Arc<AckManager>,
    leader_addr: String,
    metadata_cache: Arc<MetadataCacheManager>,
    share_sub: ShareSubShareSub,
    sx: Sender<bool>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) {
    let client_id = share_sub.client_id.clone();
    let group_name = share_sub.group_name.clone();
    let sub_name = share_sub.sub_name.clone();

    info(format!(
        "ReSub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was start successfully",
        client_id,
        group_name,
        sub_name,
    ));
    let codec = Mqtt5Codec::new();
    let socket = TcpStream::connect(leader_addr.clone()).await.unwrap();

    // split stream
    let (r_stream, w_stream) = io::split(socket);
    let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
    let mut write_frame_stream = FramedWrite::new(w_stream, codec.clone());

    let follower_sub_leader_client_id = unique_id();

    // Create a connection to GroupName
    let connect_pkg = build_resub_connect_pkg(follower_sub_leader_client_id.clone()).clone();
    match write_frame_stream.send(connect_pkg).await {
        Ok(_) => {}
        Err(e) => {
            error(format!(
                "ReSub follower for client_id:[{}], group_name:[{}], sub_name:[{}] send Connect packet error. error message:{}",  
                client_id,
                group_name,
                sub_name,
                e.to_string()
            ));
            return;
        }
    }

    // Continuously receive back packet information from the Server
    loop {
        match sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    info(format!(
                        "Rewrite sub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was stopped successfully",
                        client_id,
                        group_name,
                        sub_name,
                    ));

                    // When a thread exits, an unsubscribed mqtt packet is sent
                    let unscribe_pkg = build_resub_unsubscribe_pkg(share_sub.clone());
                    match write_frame_stream.send(unscribe_pkg).await {
                        Ok(_) => {}
                        Err(e) => {
                            error(format!(
                                "Share follower for client_id:[{}], group_name:[{}], sub_name:[{}] send UnSubscribe packet error. error message:{}",
                                client_id,
                                group_name,
                                sub_name,
                                e.to_string()
                            ));
                        }
                    }
                    break;
                }
            }
            Err(_) => {}
        }

        if let Some(data) = read_frame_stream.next().await {
            match data {
                Ok(da) => {
                    match da {
                        MQTTPacket::ConnAck(connack, connack_properties) => {
                            if connack.code == ConnectReturnCode::Success {
                                // When the connection is successful, a subscription request is sent
                                let sub_packet = build_resub_subscribe_pkg(share_sub.clone());
                                match write_frame_stream.send(sub_packet).await {
                                    Ok(_) => {
                                        continue;
                                    }
                                    Err(e) => {
                                        error(format!(
                                            "Share follower [{}] send Subscribe packet error. error message:{}",
                                            follower_sub_leader_client_id,
                                            e.to_string()
                                        ));
                                        break;
                                    }
                                }
                            } else {
                                error(
                                    format!("client_id:[{}], group_name:[{}], sub_name:[{}] Follower forwarding subscription connection request error, error message: {:?},{:?}",
                                client_id,
                                group_name,
                                sub_name,
                                connack,
                                connack_properties),
                                );
                                break;
                            }
                        }

                        MQTTPacket::SubAck(suback, suback_properties) => {
                            for reason in suback.return_codes.clone() {
                                if reason
                                    == SubscribeReasonCode::Success(
                                        protocol::mqtt::QoS::AtLeastOnce,
                                    )
                                    || reason
                                        == SubscribeReasonCode::Success(
                                            protocol::mqtt::QoS::AtMostOnce,
                                        )
                                    || reason
                                        == SubscribeReasonCode::Success(
                                            protocol::mqtt::QoS::ExactlyOnce,
                                        )
                                    || reason == SubscribeReasonCode::QoS0
                                    || reason == SubscribeReasonCode::QoS1
                                    || reason == SubscribeReasonCode::QoS2
                                {
                                    continue;
                                }
                                error(format!("client_id:[{}], group_name:[{}], sub_name:[{}]Follower forwarding subscription request error, error message: {:?},{:?}",
                                client_id,
                                group_name,
                                sub_name
                                ,suback,suback_properties));
                                break;
                            }
                        }

                        MQTTPacket::Publish(publish, publish_properties) => match publish.qos {
                            protocol::mqtt::QoS::AtMostOnce => {
                                publish_message_qos0(
                                    metadata_cache.clone(),
                                    client_id.clone(),
                                    publish,
                                    publish_properties.unwrap(),
                                    share_sub.protocol.clone(),
                                    response_queue_sx4.clone(),
                                    response_queue_sx5.clone(),
                                    sx.clone(),
                                );
                            }

                            protocol::mqtt::QoS::AtLeastOnce => {
                                let (wait_puback_sx, _) = broadcast::channel(1);
                                ack_manager.add(
                                    client_id.clone(),
                                    share_sub.packet_identifier,
                                    AckPacketInfo {
                                        sx: wait_puback_sx.clone(),
                                        create_time: now_second(),
                                    },
                                );
                                resub_publish_message_qos1(
                                    metadata_cache.clone(),
                                    client_id.clone(),
                                    publish,
                                    publish_properties.unwrap(),
                                    share_sub.packet_identifier,
                                    share_sub.protocol.clone(),
                                    response_queue_sx4.clone(),
                                    response_queue_sx5.clone(),
                                    sx.clone(),
                                    wait_puback_sx,
                                    &mut write_frame_stream,
                                );
                            }

                            protocol::mqtt::QoS::ExactlyOnce => {}
                        },

                        MQTTPacket::PubRel(pubrel, _) => {
                            // let client_id = share_sub.client_id.clone();
                            // let (qos_sx, qos_rx) = broadcast::channel(1);
                            // let pkid = 1;

                            // // send PubRel to Client
                            // let pubrel = PubRel {
                            //     pkid,
                            //     reason: PubRelReason::Success,
                            // };
                            // let pubrel_properties = PubRelProperties::default();

                            // ack_manager.add(
                            //     client_id.clone(),
                            //     pubrel.pkid,
                            //     AckPacketInfo {
                            //         sx: qos_sx,
                            //         create_time: now_second(),
                            //     },
                            // );

                            // if wait_packet_ack(qos_rx).await > 0 {
                            //     let puback =
                            //         build_publish_comp(pubrel.pkid, PubCompReason::Success);
                            //     let _ = stream.send(puback).await;
                            // }
                        }

                        MQTTPacket::Disconnect(_, _) => {
                            break;
                        }

                        MQTTPacket::UnsubAck(_, _) => {
                            break;
                        }
                        _ => {
                            error("Rewrite subscription thread cannot recognize the currently returned package".to_string());
                        }
                    }
                }
                Err(e) => error(e.to_string()),
            }
        }
    }
}

async fn resub_publish_message_qos1(
    metadata_cache: Arc<MetadataCacheManager>,
    mqtt_client_id: String,
    mut publish: Publish,
    publish_properties: PublishProperties,
    pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    wait_puback_sx: broadcast::Sender<AckPackageData>,
    write_frame_stream: &mut FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
) -> Result<(), RobustMQError> {
    let mut retry_times = 0;
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

        let connect_id = if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        retry_times = retry_times + 1;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish.clone(), Some(publish_properties.clone())),
        };

        match publish_to_response_queue(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                if let Some(data) = wait_packet_ack(wait_puback_sx.clone()).await {
                    if data.ack_type == AckPackageType::PubAck && data.pkid == pkid {
                        let puback = build_resub_publish_ack(pkid, PubAckReason::Success);
                        loop {
                            match stop_sx.subscribe().try_recv() {
                                Ok(flag) => {
                                    if flag {
                                        return Ok(());
                                    }
                                }
                                Err(_) => {}
                            }
                            match write_frame_stream.send(puback.clone()).await {
                                Ok(_) => {
                                    break;
                                }
                                Err(e) => {
                                    error(format!(
                                        "Share follower send PubAck packet error. error message:{}",
                                        e.to_string()
                                    ));
                                    sleep(Duration::from_secs(1)).await
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error(format!(
                    "Failed to write QOS1 Publish message to response queue, failure message: {}",
                    e.to_string()
                ));
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// send publish message
// wait pubrec message
// send pubrel message
// wait pubcomp message
pub async fn resub_publish_message_qos2(
    metadata_cache: Arc<MetadataCacheManager>,
    mqtt_client_id: String,
    mut publish: Publish,
    publish_properties: PublishProperties,
    pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    wait_ack_sx: broadcast::Sender<AckPackageData>,
    write_frame_stream: &mut FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
) -> Result<(), RobustMQError> {
    // send publish message
    let mut retry_times = 0;
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

        let connect_id = if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        retry_times = retry_times + 1;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish.clone(), Some(publish_properties.clone())),
        };

        match publish_to_response_queue(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                break;
            }
            Err(e) => {
                error(format!(
                    "Failed to write QOS1 Publish message to response queue, failure message: {}",
                    e.to_string()
                ));
                sleep(Duration::from_millis(5)).await;
            }
        }
    }

    // wait pub rec
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

        let connect_id = if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        if let Some(data) = wait_packet_ack(wait_ack_sx.clone()).await {
            if data.ack_type == AckPackageType::PubRec {
                let puback = build_resub_publish_rec(pkid, PubRecReason::Success, connect_id);
                loop {
                    match stop_sx.subscribe().try_recv() {
                        Ok(flag) => {
                            if flag {
                                return Ok(());
                            }
                        }
                        Err(_) => {}
                    }
                    match write_frame_stream.send(puback.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            error(format!(
                                "Share follower send PubRec packet error. error message:{}",
                                e.to_string()
                            ));
                            sleep(Duration::from_secs(1)).await
                        }
                    }
                }
                break;
            }
        }
    }

    // send pub rel
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

        let connect_id = if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
            id
        } else {
            sleep(Duration::from_secs(1)).await;
            continue;
        };

        let pubrel = PubRel {
            pkid,
            reason: protocol::mqtt::PubRelReason::Success,
        };

        let pubrel_resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::PubRel(pubrel, None),
        };

        match publish_to_response_queue(
            protocol.clone(),
            pubrel_resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                break;
            }
            Err(e) => {
                error(format!(
                    "Failed to write PubRel message to response queue, failure message: {}",
                    e.to_string()
                ));
            }
        }
    }

    // wait pub comp
    let (wait_pubcomp_sx, _) = broadcast::channel(1);
    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        if let Some(data) = wait_packet_ack(wait_pubcomp_sx.clone()).await {
            if data.ack_type == AckPackageType::PubComp {
                let pubcomp = build_resub_publish_comp(pkid, PubCompReason::Success);
                loop {
                    match stop_sx.subscribe().try_recv() {
                        Ok(flag) => {
                            if flag {
                                return Ok(());
                            }
                        }
                        Err(_) => {}
                    }
                    match write_frame_stream.send(pubcomp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            error(format!(
                                "Share follower send PubComp packet error. error message:{}",
                                e.to_string()
                            ));
                            sleep(Duration::from_secs(1)).await
                        }
                    }
                }
                break;
            }
        }
    }
    return Ok(());
}

fn build_resub_connect_pkg(client_id: String) -> MQTTPacket {
    let conf = broker_mqtt_conf();
    let connect = Connect {
        keep_alive: 60,
        client_id,
        clean_session: true,
    };

    let mut properties = ConnectProperties::default();
    properties.session_expiry_interval = Some(60);
    properties.user_properties = vec![share_sub_rewrite_publish_flag()];

    let login = Login {
        username: conf.system.system_user.clone(),
        password: conf.system.system_password.clone(),
    };

    return MQTTPacket::Connect(connect, Some(properties), None, None, Some(login));
}

fn build_resub_subscribe_pkg(share_sub: ShareSubShareSub) -> MQTTPacket {
    let subscribe = Subscribe {
        packet_identifier: share_sub.packet_identifier,
        filters: vec![share_sub.filter],
    };

    let subscribe_poperties = SubscribeProperties {
        subscription_identifier: share_sub.subscription_identifier,
        user_properties: vec![share_sub_rewrite_publish_flag()],
    };

    return MQTTPacket::Subscribe(subscribe, Some(subscribe_poperties));
}

fn build_resub_unsubscribe_pkg(rewrite_sub: ShareSubShareSub) -> MQTTPacket {
    return MQTTPacket::Unsubscribe(
        Unsubscribe::default(),
        Some(UnsubscribeProperties::default()),
    );
}

fn build_resub_publish_ack(pkid: u16, reason: PubAckReason) -> MQTTPacket {
    let puback = PubAck { pkid, reason };
    let puback_properties = PubAckProperties::default();
    return MQTTPacket::PubAck(puback, Some(puback_properties));
}

fn build_resub_publish_rec(pkid: u16, reason: PubRecReason, mqtt_client_pkid: u64) -> MQTTPacket {
    let puback = PubRec { pkid, reason };
    let puback_properties = PubRecProperties {
        reason_string: None,
        user_properties: vec![("mqtt_client_pkid".to_string(), mqtt_client_pkid.to_string())],
    };
    return MQTTPacket::PubRec(puback, Some(puback_properties));
}

fn build_resub_publish_comp(pkid: u16, reason: PubCompReason) -> MQTTPacket {
    let pubcomp = PubComp { pkid, reason };
    let pubcomp_properties = PubCompProperties::default();
    return MQTTPacket::PubComp(pubcomp, Some(pubcomp_properties));
}

#[cfg(test)]
mod tests {}
