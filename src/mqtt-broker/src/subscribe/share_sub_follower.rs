use super::{
    sub_manager::SubscribeManager,
    subscribe::{
        get_share_sub_leader, publish_to_response_queue, share_sub_rewrite_publish_flag,
        wait_packet_ack,
    },
};
use crate::{
    core::metadata_cache::MetadataCacheManager,
    qos::ack_manager::{AckManager, AckPacketInfo},
    server::{tcp::packet::ResponsePackage, MQTTProtocol},
    subscribe::sub_manager::ShareSubShareSub,
};
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info, warn},
    tools::{now_second, unique_id},
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use protocol::{
    mqtt::{
        Connect, ConnectProperties, ConnectReturnCode, Login, MQTTPacket, PubAck, PubAckProperties,
        PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec, PubRecProperties,
        PubRecReason, PubRel, PubRelProperties, PubRelReason, Publish, PublishProperties,
        Subscribe, SubscribeProperties, SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
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
    // (group_name_sub_name, Sender<bool>)
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
            for (group_name_sub_name, share_sub) in
                self.subscribe_manager.share_follower_subscribe.clone()
            {
                let metadata_cache = self.metadata_cache.clone();
                let response_queue_sx4 = self.response_queue_sx4.clone();
                let response_queue_sx5 = self.response_queue_sx5.clone();
                let (stop_sx, stop_rx) = mpsc::channel(1);
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
                            let follower_key =
                                format!("{}_{}", share_sub.group_name, share_sub.sub_name);
                            self.subscribe_manager
                                .share_follower_subscribe
                                .remove(&follower_key);

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
                            self.follower_sub_thread
                                .insert(group_name_sub_name.clone(), stop_sx);
                            tokio::spawn(async move {
                                if share_sub.protocol == MQTTProtocol::MQTT4 {
                                    resub_sub_mqtt4().await;
                                } else if share_sub.protocol == MQTTProtocol::MQTT5 {
                                    resub_sub_mqtt5(
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
            sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn resub_sub_mqtt4() {
    error("MQTT 4 does not currently support shared subscriptions".to_string());
}

async fn resub_sub_mqtt5(
    ack_manager: Arc<AckManager>,
    leader_addr: String,
    metadata_cache: Arc<MetadataCacheManager>,
    share_sub: ShareSubShareSub,
    mut rx: Receiver<bool>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) {
    info(format!(
        "ReSub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was start successfully",
        share_sub.client_id.clone(),
        share_sub.group_name.clone(),
        share_sub.sub_name.clone(),
    ));

    let socket = TcpStream::connect(leader_addr.clone()).await.unwrap();
    let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());

    let follower_sub_leader_client_id = unique_id();

    // Create a connection to GroupName
    let connect_pkg = build_rewrite_connect_pkg(follower_sub_leader_client_id.clone()).clone();
    match stream.send(connect_pkg).await {
        Ok(_) => {}
        Err(e) => {
            error(format!(
                "ReSub follower [{}] send Connect packet error. error message:{}",
                follower_sub_leader_client_id,
                e.to_string()
            ));
            return;
        }
    }

    // Continuously receive back packet information from the Server
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
                    match stream.send(unscribe_pkg).await {
                        Ok(_) => {}
                        Err(e) => {
                            error(format!(
                                "Share follower [{}] send UnSubscribe packet error. error message:{}",
                                follower_sub_leader_client_id,
                                e.to_string()
                            ));
                            break;
                        }
                    }
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
                                // When the connection is successful, a subscription request is sent
                                let sub_packet = build_rewrite_subscribe_pkg(share_sub.clone());
                                match stream.send(sub_packet).await {
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
                                error(format!("[{}] Follower forwarding subscription connection request error, error message: {:?},{:?}",
                                follower_sub_leader_client_id,connack,connack_properties));
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
                                error(format!("[{}] Follower forwarding subscription request error, error message: ** {:?},{:?}",
                                    follower_sub_leader_client_id,suback,suback_properties));
                                break;
                            }
                        }

                        MQTTPacket::Publish(publish, _) => {
                            let mqtt_client_client_id = share_sub.client_id.clone();

                            let connect_id = if let Some(connect_id) =
                                metadata_cache.get_connect_id(mqtt_client_client_id.clone())
                            {
                                connect_id
                            } else {
                                return;
                            };

                            let mut sub_id = Vec::new();
                            if let Some(id) = share_sub.subscription_identifier.clone() {
                                sub_id.push(id);
                            }

                            let pkid: u16 =
                                metadata_cache.get_available_pkid(mqtt_client_client_id.clone());

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

                            loop {
                                match publish_to_response_queue(
                                    share_sub.protocol.clone(),
                                    resp.clone(),
                                    response_queue_sx4.clone(),
                                    response_queue_sx5.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {
                                        match publish.qos {
                                            protocol::mqtt::QoS::AtMostOnce => {
                                                break;
                                            }

                                            // protocol::mqtt::QoS::AtLeastOnce
                                            protocol::mqtt::QoS::AtLeastOnce => {
                                                metadata_cache.save_pkid_info(
                                                    mqtt_client_client_id.clone(),
                                                    pkid,
                                                );

                                                let (wait_puback_sx, wait_puback_rx) =
                                                    mpsc::channel(1);

                                                ack_manager.add(
                                                    mqtt_client_client_id.clone(),
                                                    pkid,
                                                    AckPacketInfo {
                                                        sx: wait_puback_sx,
                                                        create_time: now_second(),
                                                    },
                                                );

                                                if wait_packet_ack(wait_puback_rx).await > 0 {
                                                    let puback = build_rewrite_publish_ack(
                                                        pkid,
                                                        PubAckReason::Success,
                                                    );
                                                    match stream.send(puback).await{
                                                        Ok(_) => {break;}
                                                        Err(e) => {
                                                            error(format!("Share follower [{}] send PubAck packet error. error message:{}",
                                                            follower_sub_leader_client_id,e.to_string()))
                                                        }
                                                    }
                                                }

                                                warn(format!(
                                                    "Wait for client [{}] puback timeout",
                                                    mqtt_client_client_id
                                                ));
                                            }

                                            // protocol::mqtt::QoS::ExactlyOnce
                                            protocol::mqtt::QoS::ExactlyOnce => {
                                                metadata_cache.save_pkid_info(
                                                    mqtt_client_client_id.clone(),
                                                    pkid,
                                                );

                                                let (wait_client_pubrec_sx, wait_client_pubrec_rx) =
                                                    mpsc::channel(1);

                                                ack_manager.add(
                                                    mqtt_client_client_id.clone(),
                                                    pkid,
                                                    AckPacketInfo {
                                                        sx: wait_client_pubrec_sx,
                                                        create_time: now_second(),
                                                    },
                                                );

                                                if wait_packet_ack(wait_client_pubrec_rx).await > 0
                                                {
                                                    let puback = build_to_leader_pubrec(
                                                        publish.pkid,
                                                        PubRecReason::Success,
                                                        pkid,
                                                    );
                                                    match stream.send(puback).await {
                                                        Ok(_) => {}
                                                        Err(e) => {
                                                            error(format!("Share follower [{}] send PubRec packet error.
                                                             error message:{}",follower_sub_leader_client_id,e.to_string()));
                                                            continue;
                                                        }
                                                    }

                                                    let (
                                                        wait_leader_pubrel_sx,
                                                        wait_leader_pubrec_rx,
                                                    ) = mpsc::channel(1);

                                                    if wait_packet_ack(wait_leader_pubrec_rx).await
                                                        > 0
                                                    {
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error(e.to_string());
                                    }
                                };
                            }
                        }

                        MQTTPacket::PubRel(pubrel, _) => {
                            let client_id = share_sub.client_id.clone();
                            let (qos_sx, qos_rx) = mpsc::channel(1);
                            let pkid = 1;

                            // send PubRel to Client
                            let pubrel = PubRel {
                                pkid,
                                reason: PubRelReason::Success,
                            };
                            let pubrel_properties = PubRelProperties::default();

                            ack_manager.add(
                                client_id.clone(),
                                pubrel.pkid,
                                AckPacketInfo {
                                    sx: qos_sx,
                                    create_time: now_second(),
                                },
                            );

                            if wait_packet_ack(qos_rx).await > 0 {
                                let puback =
                                    build_publish_comp(pubrel.pkid, PubCompReason::Success);
                                let _ = stream.send(puback).await;
                            }
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

fn build_rewrite_connect_pkg(client_id: String) -> MQTTPacket {
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
        username: conf.system.system_user,
        password: conf.system.system_password,
    };

    return MQTTPacket::Connect(connect, Some(properties), None, None, Some(login));
}

fn build_rewrite_subscribe_pkg(share_sub: ShareSubShareSub) -> MQTTPacket {
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

fn build_rewrite_unsubscribe_pkg(rewrite_sub: ShareSubShareSub) -> MQTTPacket {
    return MQTTPacket::Unsubscribe(
        Unsubscribe::default(),
        Some(UnsubscribeProperties::default()),
    );
}

fn build_rewrite_publish_ack(pkid: u16, reason: PubAckReason) -> MQTTPacket {
    let puback = PubAck { pkid, reason };
    let puback_properties = PubAckProperties::default();
    return MQTTPacket::PubAck(puback, Some(puback_properties));
}

fn build_to_leader_pubrec(pkid: u16, reason: PubRecReason, mqtt_client_pkid: u16) -> MQTTPacket {
    let puback = PubRec { pkid, reason };
    let puback_properties = PubRecProperties {
        reason_string: None,
        user_properties: vec![("mqtt_client_pkid".to_string(), mqtt_client_pkid.to_string())],
    };
    return MQTTPacket::PubRec(puback, Some(puback_properties));
}

fn build_publish_comp(pkid: u16, reason: PubCompReason) -> MQTTPacket {
    let pubcomp = PubComp { pkid, reason };
    let pubcomp_properties = PubCompProperties::default();
    return MQTTPacket::PubComp(pubcomp, Some(pubcomp_properties));
}

#[cfg(test)]
mod tests {}
