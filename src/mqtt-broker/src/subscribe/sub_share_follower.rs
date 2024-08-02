use super::{
    sub_common::{
        get_share_sub_leader, publish_message_qos0, publish_message_to_client, qos2_send_publish,
        qos2_send_pubrel, wait_packet_ack,
    },
    subscribe_manager::SubscribeCacheManager,
};
use crate::{
    handler::cache_manager::{
        CacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo,
    },
    server::{connection_manager::ConnectionManager, packet::ResponsePackage},
    subscribe::subscribe_manager::ShareSubShareSub,
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
use metadata_struct::mqtt::node_extend::MQTTNodeExtend;
use protocol::mqtt::{
    common::{
        Connect, ConnectProperties, ConnectReturnCode, Login, MQTTPacket, MQTTProtocol, PingReq,
        PubAck, PubAckProperties, PubAckReason, PubComp, PubCompProperties, PubCompReason, PubRec,
        PubRecProperties, PubRecReason, Publish, PublishProperties, Subscribe, SubscribeProperties,
        SubscribeReasonCode, Unsubscribe, UnsubscribeProperties,
    },
    mqttv5::codec::Mqtt5Codec,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    io,
    net::TcpStream,
    select,
    sync::broadcast::{self, Sender},
    time::sleep,
};
use tokio_util::codec::{FramedRead, FramedWrite};

#[derive(Clone)]
pub struct SubscribeShareFollower {
    pub subscribe_manager: Arc<SubscribeCacheManager>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    client_poll: Arc<ClientPool>,
}

impl SubscribeShareFollower {
    pub fn new(
        subscribe_manager: Arc<SubscribeCacheManager>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        return SubscribeShareFollower {
            subscribe_manager,
            connection_manager,
            cache_manager,
            client_poll,
        };
    }

    pub async fn start(&self) {
        loop {
            self.start_resub_thread().await;
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn start_resub_thread(&self) {
        for (follower_resub_key, share_sub) in
            self.subscribe_manager.share_follower_subscribe.clone()
        {
            let metadata_cache = self.cache_manager.clone();
            let connection_manager = self.connection_manager.clone();

            match get_share_sub_leader(self.client_poll.clone(), share_sub.group_name.clone()).await
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
                            if share_sub.protocol == MQTTProtocol::MQTT4 {
                                error(
                                    "MQTT 4 does not currently support shared subscriptions"
                                        .to_string(),
                                );
                            } else if share_sub.protocol == MQTTProtocol::MQTT5 {
                                let extend_info: MQTTNodeExtend = match serde_json::from_str::<
                                    MQTTNodeExtend,
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
                                        error(format!("Failed to obtain the Leader of GoupName from the Placement Center with error message {}",e.to_string()));
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
                                    Err(e) => error(e.to_string()),
                                }
                                subscribe_manager
                                    .share_follower_resub_thread
                                    .remove(&follower_resub_key);
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
    leader_addr: String,
    cache_manager: Arc<CacheManager>,
    share_sub: ShareSubShareSub,
    stop_sx: Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
) -> Result<(), RobustMQError> {
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
    let socket = match TcpStream::connect(leader_addr.clone()).await {
        Ok(sock) => sock,
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    };

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
        MQTTProtocol::MQTT5.into(),
        follower_sub_leader_client_id.clone(),
    )
    .clone();
    write_stream.write_frame(connect_pkg).await;

    // Continuously receive back packet information from the Server
    let mut stop_rx: broadcast::Receiver<bool> = stop_sx.subscribe();
    loop {
        select! {
            val = stop_rx.recv() =>{
                match val{
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
                            write_stream.write_frame(unscribe_pkg).await;
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }
            val = read_frame_stream.next()=>{
                if let Some(data) = val{
                    match data{
                        Ok(packet) => {
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
                        },
                        Err(_) =>{
                        }
                    }
                }
            }
        }
    }
    return Ok(());
}

async fn process_packet(
    cache_manager: &Arc<CacheManager>,
    share_sub: &ShareSubShareSub,
    stop_sx: &Sender<bool>,
    connection_manager: &Arc<ConnectionManager>,
    packet: MQTTPacket,
    write_stream: &Arc<WriteStream>,
    follower_sub_leader_pkid: u16,
    follower_sub_leader_client_id: &String,
    mqtt_client_id: &String,
    group_name: &String,
    sub_name: &String,
) -> bool {
    info(format!("sub follower recv:{:?}", packet));
    match packet {
        MQTTPacket::ConnAck(connack, connack_properties) => {
            if connack.code == ConnectReturnCode::Success {
                // start ping thread
                start_ping_thread(write_stream.clone(), stop_sx.clone()).await;

                // When the connection is successful, a subscription request is sent
                let sub_packet =
                    build_resub_subscribe_pkg(follower_sub_leader_pkid, share_sub.clone());
                write_stream.write_frame(sub_packet).await;

                return false;
            } else {
                error(format!("client_id:[{}], group_name:[{}], sub_name:[{}] Follower forwarding subscription connection request error, 
                            error message: {:?},{:?}",mqtt_client_id,group_name,sub_name,connack,connack_properties));
                return true;
            }
        }

        MQTTPacket::SubAck(suback, _) => {
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
            return !is_success;
        }

        MQTTPacket::Publish(mut publish, publish_properties) => {
            let cache_manager = cache_manager.clone();
            let stop_sx = stop_sx.clone();
            let connection_manager = connection_manager.clone();
            let write_stream = write_stream.clone();
            let follower_sub_leader_client_id = follower_sub_leader_client_id.clone();
            let mqtt_client_id = mqtt_client_id.clone();
            tokio::spawn(async move {
                match publish.qos {
                    // 1. leader publish to resub thread
                    protocol::mqtt::common::QoS::AtMostOnce => {
                        publish.dup = false;
                        publish_message_qos0(
                            &cache_manager,
                            &mqtt_client_id,
                            &publish,
                            &publish_properties,
                            &connection_manager,
                            &stop_sx,
                        )
                        .await;
                    }

                    protocol::mqtt::common::QoS::AtLeastOnce => {
                        let publish_to_client_pkid: u16 =
                            cache_manager.get_pkid(&mqtt_client_id).await;

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
                            &mqtt_client_id,
                            publish,
                            &publish_properties,
                            publish_to_client_pkid,
                            &connection_manager,
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
                                error(e.to_string());
                            }
                        }
                    }

                    protocol::mqtt::common::QoS::ExactlyOnce => {
                        let publish_to_client_pkid: u16 =
                            cache_manager.get_pkid(&mqtt_client_id).await;

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
                            &mqtt_client_id,
                            &publish,
                            publish_to_client_pkid,
                            &connection_manager,
                            &stop_sx,
                            &wait_client_ack_sx,
                            &wait_leader_ack_sx,
                            &write_stream,
                            &publish_properties,
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
                                error(e.to_string());
                            }
                        }
                    }
                }
            });

            return false;
        }

        MQTTPacket::PubRel(pubrel, _) => {
            if let Some(data) =
                cache_manager.get_ack_packet(follower_sub_leader_client_id.clone(), pubrel.pkid)
            {
                match data.sx.send(QosAckPackageData {
                    ack_type: QosAckPackageType::PubRel,
                    pkid: pubrel.pkid,
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        error(e.to_string());
                    }
                }
            }
            return false;
        }

        MQTTPacket::Disconnect(_, _) => {
            info("receive disconnect".to_string());
            return true;
        }

        MQTTPacket::UnsubAck(_, _) => {
            // When a thread exits, an unsubscribed mqtt packet is sent
            let unscribe_pkg =
                build_resub_unsubscribe_pkg(follower_sub_leader_pkid, share_sub.clone());
            write_stream.write_frame(unscribe_pkg).await;
            return true;
        }
        MQTTPacket::PingResp(_) => {
            return false;
        }
        _ => {
            error(
                "Rewrite subscription thread cannot recognize the currently returned package"
                    .to_string(),
            );
            return false;
        }
    }
}

async fn start_ping_thread(write_stream: Arc<WriteStream>, stop_sx: Sender<bool>) {
    tokio::spawn(async move {
        info("start_ping_thread start".to_string());

        loop {
            let send_ping = async {
                let ping_packet = MQTTPacket::PingReq(PingReq {});
                write_stream.write_frame(ping_packet.clone()).await;
                sleep(Duration::from_secs(20)).await;
            };

            let mut stop_rx = stop_sx.subscribe();
            select! {
                val = stop_rx.recv() => {
                    match val{
                        Ok(flag) => {
                            if flag {
                                info("start_ping_thread stop".to_string());
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                },
                _ = send_ping => {}

            }
        }
    });
}

async fn resub_publish_message_qos1(
    metadata_cache: &Arc<CacheManager>,
    mqtt_client_id: &String,
    mut publish: Publish,
    properties: &Option<PublishProperties>,
    publish_to_client_pkid: u16,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
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

        let connect_id = if let Some(id) = metadata_cache.get_connect_id(&mqtt_client_id) {
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

        retry_times = retry_times + 1;
        publish.pkid = publish_to_client_pkid;
        publish.dup = retry_times >= 2;

        let resp = ResponsePackage {
            connection_id: connect_id,
            packet: MQTTPacket::Publish(publish.clone(), properties.clone()),
        };

        // 2. publish to mqtt client
        match publish_message_to_client(resp, connection_manager).await {
            Ok(_) => {
                // 3. wait mqtt client puback
                if let Some(data) = wait_packet_ack(wait_puback_sx).await {
                    if data.ack_type == QosAckPackageType::PubAck
                        && data.pkid == publish_to_client_pkid
                    {
                        // 4. puback message to sub leader
                        let puback =
                            build_resub_publish_ack(current_message_pkid, PubAckReason::Success);

                        info(format!("send leader:{:?}", puback));
                        write_stream.write_frame(puback.clone()).await;
                        return Ok(());
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

// 1. send publish message
// 2. wait pubrec message
// 3. send pubrel message
// 4. wait pubcomp message
pub async fn resub_publish_message_qos2(
    metadata_cache: &Arc<CacheManager>,
    mqtt_client_id: &String,
    publish: &Publish,
    publish_to_client_pkid: u16,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
    wait_client_ack_sx: &broadcast::Sender<QosAckPackageData>,
    wait_leader_ack_sx: &broadcast::Sender<QosAckPackageData>,
    write_stream: &Arc<WriteStream>,
    publish_properties: &Option<PublishProperties>,
) -> Result<(), RobustMQError> {
    let current_message_pkid = publish.pkid;

    // 2. Send publish message to mqtt client
    match qos2_send_publish(
        connection_manager,
        metadata_cache,
        mqtt_client_id,
        publish,
        publish_properties,
        stop_sx,
    )
    .await
    {
        Ok(()) => {}
        Err(e) => return Err(e),
    };

    let mut stop_rx = stop_sx.subscribe();
    loop {
        select! {
            val = stop_rx.recv() => {
                match val{
                    Ok(flag) => {
                        if flag {
                            return Ok(());
                        }
                    }
                    Err(_) => {}
                }
            }
            // 3. Wait mqtt client PubRec
            val = wait_packet_ack(wait_client_ack_sx) => {
                if let Some(data) = val{

                    if data.ack_type == QosAckPackageType::PubRec && data.pkid == publish_to_client_pkid {
                        // 4. pubrec to leader
                        let connect_id =
                            if let Some(id) = metadata_cache.get_connect_id(&mqtt_client_id) {
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

                        info(format!("send leader:{:?}", pubrec));
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
                match val{
                    Ok(flag) => {
                        if flag {
                            return Ok(());
                        }
                    }
                    Err(_) => {}
                }
            }

            // 5. Wait Sub leader PubRel
            val =  wait_packet_ack(wait_leader_ack_sx) =>{
                if let Some(data) = val{
                    if data.ack_type == QosAckPackageType::PubRel && data.pkid == current_message_pkid {
                        // 6. Send PubRel to mqtt client
                        qos2_send_pubrel(
                            metadata_cache,
                            mqtt_client_id,
                            publish_to_client_pkid,
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
                match val{
                    Ok(flag) => {
                        if flag {
                            return Ok(());
                        }
                    }
                    Err(_) => {}
                }
            }
            val = wait_packet_ack(wait_client_ack_sx) => {
                if let Some(data) = val{
                    if data.ack_type == QosAckPackageType::PubComp {
                        // 8.pubcomp to leader
                        let pubcomp =
                            build_resub_publish_comp(current_message_pkid, PubCompReason::Success);

                        info(format!("send leader:{:?}", pubcomp));
                        write_stream.write_frame(pubcomp).await;
                        break;
                    }
                }
            }
        }
    }
    return Ok(());
}

fn build_resub_connect_pkg(protocol_level: u8, client_id: String) -> MQTTPacket {
    let conf = broker_mqtt_conf();
    let connect = Connect {
        keep_alive: 6000,
        client_id,
        clean_session: true,
    };

    let mut properties = ConnectProperties::default();
    properties.session_expiry_interval = Some(60);
    properties.user_properties = Vec::new();

    let login = Login {
        username: conf.system.default_user.clone(),
        password: conf.system.default_password.clone(),
    };

    return MQTTPacket::Connect(
        protocol_level,
        connect,
        Some(properties),
        None,
        None,
        Some(login),
    );
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
        user_properties: Vec::new(),
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
            user_properties: Vec::new(),
        }),
    );
}

fn build_resub_publish_ack(pkid: u16, reason: PubAckReason) -> MQTTPacket {
    let puback = PubAck {
        pkid,
        reason: Some(reason),
    };
    let puback_properties = PubAckProperties::default();
    return MQTTPacket::PubAck(puback, Some(puback_properties));
}

fn build_resub_publish_rec(pkid: u16, reason: PubRecReason, mqtt_client_pkid: u64) -> MQTTPacket {
    let puback = PubRec {
        pkid,
        reason: Some(reason),
    };
    let puback_properties = PubRecProperties {
        reason_string: None,
        user_properties: vec![("mqtt_client_pkid".to_string(), mqtt_client_pkid.to_string())],
    };
    return MQTTPacket::PubRec(puback, Some(puback_properties));
}

fn build_resub_publish_comp(pkid: u16, reason: PubCompReason) -> MQTTPacket {
    let pubcomp = PubComp {
        pkid,
        reason: Some(reason),
    };
    let pubcomp_properties = PubCompProperties::default();
    return MQTTPacket::PubComp(pubcomp, Some(pubcomp_properties));
}

pub struct WriteStream {
    write_list:
        DashMap<String, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>>,
    key: String,
    stop_sx: Sender<bool>,
}

impl WriteStream {
    pub fn new(stop_sx: Sender<bool>) -> Self {
        return WriteStream {
            key: "default".to_string(),
            write_list: DashMap::with_capacity(2),
            stop_sx,
        };
    }

    pub fn add_write(
        &self,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
    ) {
        self.write_list.insert(self.key.clone(), write);
    }

    pub async fn write_frame(&self, resp: MQTTPacket) {
        loop {
            match self.stop_sx.subscribe().try_recv() {
                Ok(flag) => {
                    if flag {
                        break;
                    }
                }
                Err(_) => {}
            }

            match self.write_list.try_get_mut(&self.key) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            error(format!(
                                "Resub Client Failed to write data to the response queue, error message: {:?}",
                                e
                            ));
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    error(format!("Resub Client [write_frame]Connection management could not obtain an available connection."));
                }
                dashmap::try_result::TryResult::Locked => {
                    error(format!("Resub Client [write_frame]Connection management failed to get connection variable reference"));
                }
            }
            sleep(Duration::from_secs(1)).await
        }
    }
}

#[cfg(test)]
mod tests {}
