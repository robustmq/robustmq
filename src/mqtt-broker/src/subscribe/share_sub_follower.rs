use super::{
    sub_manager::SubscribeManager,
    subscribe::{
        get_share_sub_leader, is_contain_rewrite_flag, publish_to_client, retry_publish,
        share_sub_rewrite_publish_flag, wait_qos_ack,
    },
};
use crate::{
    core::metadata_cache::MetadataCacheManager,
    qos::ack_manager::{self, AckManager, AckPacketInfo},
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
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use protocol::{
    mqtt::{
        Connect, ConnectProperties, ConnectReturnCode, Login, MQTTPacket, PubComp,
        PubCompProperties, PubRec, PubRecProperties, Publish, PublishProperties, QoS, Subscribe,
        SubscribeProperties, SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
    },
    mqttv5::codec::Mqtt5Codec,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{
        broadcast,
        mpsc::{self, Receiver, Sender},
    },
    time::sleep,
};
use tokio_util::codec::Framed;

#[derive(Clone)]
pub struct SubscribeShareFollower {
    // (client_id, Sender<bool>)
    pub follower_sub_thread: DashMap<String, Sender<bool>>,
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
            follower_sub_thread: DashMap::with_capacity(128),
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
            for (client_id, share_sub) in self.subscribe_manager.share_follower_subscribe.clone() {
                let metadata_cache = self.metadata_cache.clone();
                let response_queue_sx4 = self.response_queue_sx4.clone();
                let response_queue_sx5 = self.response_queue_sx5.clone();
                let (stop_sx, stop_rx) = mpsc::channel(1);
                self.follower_sub_thread.insert(client_id.clone(), stop_sx);
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
                                .remove(&client_id);

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
                            tokio::spawn(async move {
                                if share_sub.protocol == MQTTProtocol::MQTT4 {
                                    rewrite_sub_mqtt4().await;
                                } else if share_sub.protocol == MQTTProtocol::MQTT5 {
                                    rewrite_sub_mqtt5(
                                        ack_manager,
                                        reply.broker_ip,
                                        metadata_cache,
                                        share_sub,
                                        stop_rx,
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
            sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn rewrite_sub_mqtt4() {
    error("MQTT 4 does not currently support shared subscriptions".to_string());
}

async fn rewrite_sub_mqtt5(
    ack_manager: Arc<AckManager>,
    leader_addr: String,
    metadata_cache: Arc<MetadataCacheManager>,
    share_sub: ShareSubShareSub,
    mut rx: Receiver<bool>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) {
    info(format!(
        "Rewrite sub mqtt5 thread for client [{}] was start successfully",
        share_sub.client_id.clone()
    ));
    let socket = TcpStream::connect(leader_addr.clone()).await.unwrap();
    let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());
    let client_id = unique_id();
    let connect_pkg = build_rewrite_connect_pkc(client_id);
    let _ = stream.send(connect_pkg.clone()).await;
    loop {
        match rx.try_recv() {
            Ok(flag) => {
                if flag {
                    info(format!(
                        "Rewrite sub mqtt5 thread for client [{}] was stopped successfully",
                        share_sub.client_id.clone()
                    ));

                    // When a thread exits, an unsubscribed mqtt packet is sent
                    let unscribe_pkg = build_rewrite_unsubscribe_pkg(share_sub.clone());
                    let _ = stream.send(unscribe_pkg).await;
                }
            }
            Err(_) => {}
        }
        if let Some(data) = stream.next().await {
            match data {
                Ok(da) => {
                    match da {
                        MQTTPacket::ConnAck(connack, connack_properties) => {
                            if connack.code == ConnectReturnCode::Success {
                                let sub_packet = build_rewrite_subscribe_pkg(share_sub.clone());
                                _ = stream.send(sub_packet).await;
                            } else {
                                error(format!("Follower forwarding subscription connection request error, error message: {:?},{:?}",connack,connack_properties));
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
                                error(format!("Follower forwarding subscription request error, error message: ** {:?},{:?}",suback,suback_properties));
                                break;
                            }
                        }

                        MQTTPacket::Publish(publish, _) => {
                            packet_publish(
                                metadata_cache.clone(),
                                ack_manager.clone(),
                                share_sub.clone(),
                                publish,
                                response_queue_sx4.clone(),
                                response_queue_sx5.clone(),
                            )
                            .await;
                        }

                        MQTTPacket::PubRec(pubrec, pubrec_properties) => {
                            if let Some(properties) = pubrec_properties.clone() {
                                if is_contain_rewrite_flag(properties.user_properties) {
                                    packet_pubrec(pubrec, pubrec_properties).await;
                                }
                            }
                            // Subscribe data to Sub Leader node and send subscription request packet
                            let packet = build_rewrite_subscribe_pkg(share_sub.clone());
                        }

                        MQTTPacket::PubComp(pubcomp, pubcomp_properties) => {
                            if let Some(properties) = pubcomp_properties.clone() {
                                if is_contain_rewrite_flag(properties.user_properties) {
                                    packet_pubcomp(pubcomp, pubcomp_properties).await;
                                }
                            }
                        }

                        MQTTPacket::Disconnect(_, _) => {
                            break;
                        }

                        MQTTPacket::UnsubAck(unsuback, unsuback_properties) => {
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

async fn packet_publish(
    metadata_cache: Arc<MetadataCacheManager>,
    ack_manager: Arc<AckManager>,
    share_sub: ShareSubShareSub,
    publish: Publish,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) {
    let client_id = share_sub.client_id;

    let connect_id = if let Some(connect_id) = metadata_cache.get_connect_id(client_id.clone()) {
        connect_id
    } else {
        return;
    };

    let mut sub_id = Vec::new();
    if let Some(id) = share_sub.subscription_identifier.clone() {
        sub_id.push(id);
    }

    let pkid: u16 = metadata_cache.get_available_pkid(client_id.clone());

    let client_pub = Publish {
        dup: false,
        qos: publish.qos,
        pkid,
        retain: false,
        topic: publish.topic.clone(),
        payload: publish.payload.clone(),
    };

    let properties = PublishProperties {
        payload_format_indicator: None,
        message_expiry_interval: None,
        topic_alias: None,
        response_topic: None,
        correlation_data: None,
        user_properties: Vec::new(),
        subscription_identifiers: sub_id.clone(),
        content_type: None,
    };

    let resp = ResponsePackage {
        connection_id: connect_id,
        packet: MQTTPacket::Publish(client_pub, Some(properties)),
    };

    match retry_publish(
        client_id,
        pkid,
        metadata_cache,
        ack_manager,
        publish.qos,
        share_sub.protocol.clone(),
        resp,
        response_queue_sx4.clone(),
        response_queue_sx5.clone(),
    )
    .await
    {
        Ok(()) => {}
        Err(e) => error(e.to_string()),
    }
}

async fn packet_pubrec(publish: PubRec, pubrec_properties: Option<PubRecProperties>) {}

async fn packet_pubcomp(publish: PubComp, pubcomp_properties: Option<PubCompProperties>) {}

pub fn build_rewrite_connect_pkc(client_id: String) -> MQTTPacket {
    let connect = Connect {
        keep_alive: 60,
        client_id,
        clean_session: true,
    };

    let mut properties = ConnectProperties::default();
    properties.session_expiry_interval = Some(60);
    properties.user_properties = vec![share_sub_rewrite_publish_flag()];
    let login = Login::default();
    return MQTTPacket::Connect(connect, Some(properties), None, None, Some(login));
}

// build subscribe pkg
pub fn build_rewrite_subscribe_pkg(share_sub: ShareSubShareSub) -> MQTTPacket {
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

pub fn build_rewrite_unsubscribe_pkg(rewrite_sub: ShareSubShareSub) -> MQTTPacket {
    return MQTTPacket::Unsubscribe(
        Unsubscribe::default(),
        Some(UnsubscribeProperties::default()),
    );
}

pub async fn retry_publish_and_rewrite(
    client_id: String,
    pkid: u16,
    metadata_cache: Arc<MetadataCacheManager>,
    ack_manager: Arc<AckManager>,
    qos: QoS,
    protocol: MQTTProtocol,
    resp: ResponsePackage,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) -> Result<(), RobustMQError> {
    loop {
        match publish_to_client(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                match qos {
                    protocol::mqtt::QoS::AtMostOnce => {
                        return Ok(());
                    }
                    // protocol::mqtt::QoS::AtLeastOnce
                    protocol::mqtt::QoS::AtLeastOnce => {
                        metadata_cache.save_pkid_info(client_id.clone(), pkid);
                        let (qos_sx, qos_rx) = mpsc::channel(1);
                        ack_manager.add(
                            client_id.clone(),
                            pkid,
                            AckPacketInfo {
                                sx: qos_sx,
                                create_time: now_second(),
                            },
                        );

                        if wait_qos_ack(qos_rx).await {
                            
                            return Ok(());
                        }
                    }
                    // protocol::mqtt::QoS::ExactlyOnce
                    protocol::mqtt::QoS::ExactlyOnce => {
                        metadata_cache.save_pkid_info(client_id.clone(), pkid);

                        let (qos_sx, qos_rx) = mpsc::channel(1);
                        ack_manager.add(
                            client_id.clone(),
                            pkid,
                            AckPacketInfo {
                                sx: qos_sx,
                                create_time: now_second(),
                            },
                        );
                    }
                }
            }
            Err(e) => {
                error(e.to_string());
            }
        };
    }
}

#[cfg(test)]
mod tests {}
