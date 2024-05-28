use super::{
    sub_exclusive::publish_message_qos0,
    sub_manager::SubscribeManager,
    sub_common::{
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
        PubRecReason, PubRel, Publish, Subscribe, SubscribeProperties, SubscribeReasonCode,
        Unsubscribe, UnsubscribeProperties,
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
                            .insert(follower_resub_key.clone(), stop_sx.clone());
                        let subscribe_manager = self.subscribe_manager.clone();

                        tokio::spawn(async move {
                            if share_sub.protocol == MQTTProtocol::MQTT4 {
                                error(
                                    "MQTT 4 does not currently support shared subscriptions"
                                        .to_string(),
                                );
                            } else if share_sub.protocol == MQTTProtocol::MQTT5 {
                                resub_sub_mqtt5(
                                    follower_resub_key,
                                    ack_manager,
                                    reply.broker_ip,
                                    metadata_cache,
                                    share_sub,
                                    stop_sx,
                                    subscribe_manager,
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
    follower_resub_key: String,
    ack_manager: Arc<AckManager>,
    leader_addr: String,
    metadata_cache: Arc<MetadataCacheManager>,
    share_sub: ShareSubShareSub,
    sx: Sender<bool>,
    subscribe_manager: Arc<SubscribeManager>,
    response_queue_sx4: broadcast::Sender<ResponsePackage>,
    response_queue_sx5: broadcast::Sender<ResponsePackage>,
) {
    let mqtt_client_id = share_sub.client_id.clone();
    let group_name = share_sub.group_name.clone();
    let sub_name = share_sub.sub_name.clone();

    info(format!(
        "ReSub mqtt5 thread for client_id:[{}], group_name:[{}], sub_name:[{}] was start successfully",
        mqtt_client_id,
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
    let follower_sub_leader_pkid: u16 = 1;

    // Create a connection to GroupName
    let connect_pkg = build_resub_connect_pkg(follower_sub_leader_client_id.clone()).clone();
    match write_frame_stream.send(connect_pkg).await {
        Ok(_) => {}
        Err(e) => {
            error(format!(
                "ReSub follower for client_id:[{}], group_name:[{}], sub_name:[{}] send Connect packet error. error message:{}",  
                mqtt_client_id,
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
                        mqtt_client_id,
                        group_name,
                        sub_name,
                    ));

                    // When a thread exits, an unsubscribed mqtt packet is sent
                    let unscribe_pkg =
                        build_resub_unsubscribe_pkg(follower_sub_leader_pkid, share_sub.clone());

                    match write_frame_stream.send(unscribe_pkg).await {
                        Ok(_) => {}
                        Err(e) => {
                            error(format!(
                                "Share follower for client_id:[{}], group_name:[{}], sub_name:[{}] send UnSubscribe packet error. error message:{}",
                                mqtt_client_id,
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
                                let sub_packet = build_resub_subscribe_pkg(
                                    follower_sub_leader_pkid,
                                    share_sub.clone(),
                                );
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
                                mqtt_client_id,
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
                                mqtt_client_id,
                                group_name,
                                sub_name
                                ,suback,suback_properties));
                                break;
                            }
                        }

                        MQTTPacket::Publish(mut publish, publish_properties) => match publish.qos {
                            // 1. leader publish to resub thread
                            protocol::mqtt::QoS::AtMostOnce => {
                                publish.dup = false;
                                publish_message_qos0(
                                    metadata_cache.clone(),
                                    mqtt_client_id.clone(),
                                    publish,
                                    publish_properties.unwrap(),
                                    share_sub.protocol.clone(),
                                    response_queue_sx4.clone(),
                                    response_queue_sx5.clone(),
                                    sx.clone(),
                                )
                                .await;
                            }

                            protocol::mqtt::QoS::AtLeastOnce => {
                                let publish_to_client_pkid: u16 =
                                    metadata_cache.get_pkid(mqtt_client_id.clone()).await;

                                let (wait_puback_sx, _) = broadcast::channel(1);
                                ack_manager.add(
                                    mqtt_client_id.clone(),
                                    publish_to_client_pkid,
                                    AckPacketInfo {
                                        sx: wait_puback_sx.clone(),
                                        create_time: now_second(),
                                    },
                                );

                                match resub_publish_message_qos1(
                                    metadata_cache.clone(),
                                    mqtt_client_id.clone(),
                                    publish,
                                    publish_to_client_pkid,
                                    share_sub.protocol.clone(),
                                    response_queue_sx4.clone(),
                                    response_queue_sx5.clone(),
                                    sx.clone(),
                                    wait_puback_sx,
                                    &mut write_frame_stream,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        metadata_cache.remove_pkid_info(
                                            mqtt_client_id.clone(),
                                            publish_to_client_pkid,
                                        );
                                        ack_manager
                                            .remove(mqtt_client_id.clone(), publish_to_client_pkid);
                                        continue;
                                    }
                                    Err(e) => {
                                        error(e.to_string());
                                    }
                                }
                            }

                            protocol::mqtt::QoS::ExactlyOnce => {
                                let publish_to_client_pkid: u16 =
                                    metadata_cache.get_pkid(mqtt_client_id.clone()).await;

                                let (wait_client_ack_sx, _) = broadcast::channel(1);

                                ack_manager.add(
                                    mqtt_client_id.clone(),
                                    publish_to_client_pkid,
                                    AckPacketInfo {
                                        sx: wait_client_ack_sx.clone(),
                                        create_time: now_second(),
                                    },
                                );

                                let (wait_leader_ack_sx, _) = broadcast::channel(1);
                                ack_manager.add(
                                    follower_sub_leader_client_id.clone(),
                                    publish.pkid,
                                    AckPacketInfo {
                                        sx: wait_leader_ack_sx.clone(),
                                        create_time: now_second(),
                                    },
                                );

                                match resub_publish_message_qos2(
                                    metadata_cache.clone(),
                                    mqtt_client_id.clone(),
                                    publish.clone(),
                                    publish_to_client_pkid,
                                    share_sub.protocol.clone(),
                                    response_queue_sx4.clone(),
                                    response_queue_sx5.clone(),
                                    sx.clone(),
                                    wait_client_ack_sx,
                                    wait_leader_ack_sx,
                                    &mut write_frame_stream,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        metadata_cache.remove_pkid_info(
                                            mqtt_client_id.clone(),
                                            publish_to_client_pkid,
                                        );
                                        ack_manager
                                            .remove(mqtt_client_id.clone(), publish_to_client_pkid);

                                        ack_manager.remove(
                                            follower_sub_leader_client_id.clone(),
                                            publish.pkid,
                                        );
                                        continue;
                                    }
                                    Err(e) => {
                                        error(e.to_string());
                                    }
                                }
                            }
                        },

                        MQTTPacket::PubRel(pubrel, _) => {
                            if let Some(data) =
                                ack_manager.get(follower_sub_leader_client_id.clone(), pubrel.pkid)
                            {
                                match data.sx.send(AckPackageData {
                                    ack_type: AckPackageType::PubRel,
                                    pkid: pubrel.pkid,
                                }) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error(e.to_string());
                                    }
                                }
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
    subscribe_manager
        .share_follower_resub_thread
        .remove(&follower_resub_key);
}

async fn resub_publish_message_qos1(
    metadata_cache: Arc<MetadataCacheManager>,
    mqtt_client_id: String,
    mut publish: Publish,
    publish_to_client_pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    wait_puback_sx: broadcast::Sender<AckPackageData>,
    write_frame_stream: &mut FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
) -> Result<(), RobustMQError> {
    let mut retry_times = 0;
    let current_message_pkid = publish.pkid;
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
        publish.pkid = publish_to_client_pkid;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish.clone(), None),
        };

        // 2. publish to mqtt client
        match publish_to_response_queue(
            protocol.clone(),
            resp.clone(),
            response_queue_sx4.clone(),
            response_queue_sx5.clone(),
        )
        .await
        {
            Ok(_) => {
                // 3. wait mqtt client puback
                if let Some(data) = wait_packet_ack(wait_puback_sx.clone()).await {
                    if data.ack_type == AckPackageType::PubAck
                        && data.pkid == publish_to_client_pkid
                    {
                        // 4. puback message to sub leader
                        let puback =
                            build_resub_publish_ack(current_message_pkid, PubAckReason::Success);
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
    publish_to_client_pkid: u16,
    protocol: MQTTProtocol,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    wait_client_ack_sx: broadcast::Sender<AckPackageData>,
    wait_leader_ack_sx: broadcast::Sender<AckPackageData>,
    write_frame_stream: &mut FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
) -> Result<(), RobustMQError> {
    let mut retry_times = 0;
    let current_message_pkid = publish.pkid;
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
        publish.pkid = publish_to_client_pkid;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish.clone(), None),
        };

        // 2. publish to client
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

    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

        // 3. wait mqtt client pubrec
        if let Some(data) = wait_packet_ack(wait_client_ack_sx.clone()).await {
            if data.ack_type == AckPackageType::PubRec && data.pkid == publish_to_client_pkid {
                // 4. pubrec to leader
                let connect_id =
                    if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
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

                loop {
                    match stop_sx.subscribe().try_recv() {
                        Ok(flag) => {
                            if flag {
                                return Ok(());
                            }
                        }
                        Err(_) => {}
                    }
                    match write_frame_stream.send(pubrec.clone()).await {
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

    loop {
        match stop_sx.subscribe().try_recv() {
            Ok(flag) => {
                if flag {
                    return Ok(());
                }
            }
            Err(_) => {}
        }

        // 5. wait leader pubrel
        if let Some(data) = wait_packet_ack(wait_leader_ack_sx.clone()).await {
            if data.ack_type == AckPackageType::PubRel && data.pkid == current_message_pkid {
                let connect_id =
                    if let Some(id) = metadata_cache.get_connect_id(mqtt_client_id.clone()) {
                        id
                    } else {
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    };

                let pubrel = PubRel {
                    pkid: publish_to_client_pkid,
                    reason: protocol::mqtt::PubRelReason::Success,
                };

                let pubrel_resp = ResponsePackage {
                    connection_id: connect_id,
                    packet: MQTTPacket::PubRel(pubrel, None),
                };
                loop {
                    match stop_sx.subscribe().try_recv() {
                        Ok(flag) => {
                            if flag {
                                return Ok(());
                            }
                        }
                        Err(_) => {}
                    }
                    // 6. pubrel to mqtt client
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
                            sleep(Duration::from_secs(1)).await
                        }
                    }
                }
                break;
            }
        }
    }

    // 7. wait client pubcomp
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
                // 8.pubcomp to leader
                let pubcomp =
                    build_resub_publish_comp(current_message_pkid, PubCompReason::Success);
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

fn build_resub_subscribe_pkg(
    follower_sub_leader_pkid: u16,
    share_sub: ShareSubShareSub,
) -> MQTTPacket {
    let subscribe = Subscribe {
        packet_identifier: follower_sub_leader_pkid,
        filters: vec![share_sub.filter],
    };

    let subscribe_poperties = SubscribeProperties {
        subscription_identifier: share_sub.subscription_identifier,
        user_properties: vec![share_sub_rewrite_publish_flag()],
    };

    return MQTTPacket::Subscribe(subscribe, Some(subscribe_poperties));
}

fn build_resub_unsubscribe_pkg(
    follower_sub_leader_pkid: u16,
    rewrite_sub: ShareSubShareSub,
) -> MQTTPacket {
    return MQTTPacket::Unsubscribe(
        Unsubscribe {
            pkid: follower_sub_leader_pkid,
            filters: vec![rewrite_sub.filter.path],
        },
        Some(UnsubscribeProperties {
            user_properties: vec![share_sub_rewrite_publish_flag()],
        }),
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
