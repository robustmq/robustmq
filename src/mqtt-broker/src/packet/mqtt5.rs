use super::packet::MQTTAckBuild;
use crate::{
    heartbeat::heartbeat_manager::{ConnectionLiveTime, HeartbeatManager},
    metadata::{
        cache::MetadataCache,
        message::Message,
        session::{LastWillData, Session},
        subscriber::Subscriber,
        topic::Topic,
    },
    storage::{message::MessageStorage, session::SessionStorage, topic::TopicStorage},
    subscribe::subscribe_manager::SubScribeManager,
};
use common_base::{
    log::error,
    tools::{now_second, unique_id_string},
};
use protocol::mqtt::{
    Connect, ConnectProperties, Disconnect, DisconnectProperties, DisconnectReasonCode, LastWill,
    LastWillProperties, Login, MQTTPacket, PingReq, PubAck, PubAckProperties, Publish,
    PublishProperties, Subscribe, SubscribeProperties, Unsubscribe, UnsubscribeProperties,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Mqtt5Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    subscribe_manager: Arc<RwLock<SubScribeManager>>,
    message_storage: MessageStorage,
    ack_build: MQTTAckBuild,
    heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
}

impl Mqtt5Service {
    pub fn new(
        metadata_cache: Arc<RwLock<MetadataCache>>,
        subscribe_manager: Arc<RwLock<SubScribeManager>>,
        ack_build: MQTTAckBuild,
        heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
    ) -> Self {
        let message_storage = MessageStorage::new();
        return Mqtt5Service {
            metadata_cache,
            subscribe_manager,
            message_storage,
            ack_build,
            heartbeat_manager,
        };
    }

    pub async fn connect(
        &mut self,
        connect_id: u64,
        connnect: Connect,
        connect_properties: Option<ConnectProperties>,
        last_will: Option<LastWill>,
        last_will_properties: Option<LastWillProperties>,
        login: Option<Login>,
    ) -> MQTTPacket {
        // connect for authentication
        if self.authentication(login).await {
            return self.ack_build.distinct(DisconnectReasonCode::NotAuthorized);
        }

        // build session data
        let mut auto_client_id = false;
        let mut client_id = connnect.client_id.clone();
        if client_id.is_empty() {
            client_id = unique_id_string();
            auto_client_id = true;
        }
        let mut session = Session::default();

        let cache = self.metadata_cache.read().await;
        let cluster = cache.cluster_info.clone();
        drop(cache);
        session = session.build_session(
            client_id.clone(),
            connnect.clone(),
            connect_properties.clone(),
            cluster.server_keep_alive(),
        );

        // save last will data
        if !last_will.is_none() {
            let last_will = LastWillData {
                last_will,
                last_will_properties,
            };
            match self
                .message_storage
                .save_lastwill(client_id.clone(), last_will)
            {
                Ok(_) => {}
                Err(e) => {
                    error(e.to_string());
                    return self
                        .ack_build
                        .distinct(DisconnectReasonCode::UnspecifiedError);
                }
            }
            session.last_will = true;
        }

        // save client session
        let session_storage = SessionStorage::new();
        match session_storage.save_session(client_id.clone(), session.clone()) {
            Ok(_) => {}
            Err(e) => {
                error(e.to_string());
                return self
                    .ack_build
                    .distinct(DisconnectReasonCode::UnspecifiedError);
            }
        }

        // update cache
        // When working with locks, the default is to release them as soon as possible
        let mut cache = self.metadata_cache.write().await;
        cache.set_session(client_id.clone(), session.clone());
        cache.set_client_id(connect_id, client_id.clone());
        drop(cache);

        let mut heartbeat = self.heartbeat_manager.write().await;
        let session_keep_alive = session.keep_alive;
        let live_time = ConnectionLiveTime {
            protobol: crate::server::MQTTProtocol::MQTT5,
            keep_live: session_keep_alive,
            heartbeat: now_second(),
        };
        heartbeat.report_hearbeat(connect_id, live_time);
        drop(heartbeat);

        let mut user_properties = Vec::new();
        if let Some(connect_properties) = connect_properties {
            user_properties = connect_properties.user_properties;
        }

        let reason_string = Some("".to_string());
        let response_information = Some("".to_string());
        let server_reference = Some("".to_string());
        return self
            .ack_build
            .conn_ack(
                client_id,
                auto_client_id,
                reason_string,
                user_properties,
                response_information,
                server_reference,
                session_keep_alive,
            )
            .await;
    }

    pub async fn publish(
        &self,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> MQTTPacket {
        let topic_name = String::from_utf8(publish.topic.to_vec()).unwrap();
        let topic = Topic::new(&topic_name);

        // Update the cache if the Topic doesn't exist and persist the Topic information
        let mut cache = self.metadata_cache.write().await;
        if !cache.topic_exists(&topic_name) {
            cache.set_topic(&topic_name, &topic);

            let topic_storage = TopicStorage::new();
            match topic_storage.save_topic(&topic_name, &topic) {
                Ok(_) => {}
                Err(e) => {
                    error(e.to_string());
                    return self
                        .ack_build
                        .distinct(DisconnectReasonCode::UnspecifiedError);
                }
            }
        }

        // Persisting stores message data
        let message = Message::build_message(publish.clone(), publish_properties.clone());
        let message_id;
        match self
            .message_storage
            .append_topic_message(topic.topic_id, message)
        {
            Ok(da) => {
                message_id = da;
            }
            Err(e) => {
                error(e.to_string());
                return self
                    .ack_build
                    .distinct(DisconnectReasonCode::UnspecifiedError);
            }
        }

        // Pub Ack information is built
        let pkid = publish.pkid;
        let mut user_properties = Vec::new();
        if let Some(properties) = publish_properties {
            user_properties = properties.user_properties;
        }
        user_properties.push(("message_id".to_string(), message_id.to_string()));

        return self.ack_build.pub_ack(pkid, None, user_properties);
    }

    pub fn publish_ack(&self, pub_ack: PubAck, puback_properties: Option<PubAckProperties>) {}

    pub async fn subscribe(
        &self,
        connect_id: u64,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) -> MQTTPacket {
        let subscriber = Subscriber::build_subscriber(
            connect_id,
            subscribe.clone(),
            subscribe_properties.clone(),
        );

        // Saving subscriptions
        let mut sub_manager = self.subscribe_manager.write().await;
        sub_manager.add_subscribe(connect_id, subscriber);
        drop(sub_manager);

        let pkid = subscribe.packet_identifier;
        let mut user_properties = Vec::new();
        if let Some(properties) = subscribe_properties {
            user_properties = properties.user_properties;
        }
        return self.ack_build.sub_ack(pkid, None, user_properties);
    }

    pub async fn ping(&self, connect_id: u64, _: PingReq) -> MQTTPacket {
        let cache = self.metadata_cache.read().await;
        if let Some(client_id) = cache.connect_id_info.get(&connect_id) {
            if let Some(session_info) = cache.session_info.get(client_id) {
                let live_time = ConnectionLiveTime {
                    protobol: crate::server::MQTTProtocol::MQTT5,
                    keep_live: session_info.keep_alive,
                    heartbeat: now_second(),
                };
                let mut heartbeat = self.heartbeat_manager.write().await;
                heartbeat.report_hearbeat(connect_id, live_time);
                return self.ack_build.ping_resp();
            }
        }
        return self
            .ack_build
            .distinct(DisconnectReasonCode::UseAnotherServer);
    }

    pub async fn un_subscribe(
        &self,
        connect_id: u64,
        un_subscribe: Unsubscribe,
        un_subscribe_properties: Option<UnsubscribeProperties>,
    ) -> MQTTPacket {
        // Remove subscription information
        let mut sub_manager = self.subscribe_manager.write().await;
        sub_manager.remove_subscribe(connect_id, Some(un_subscribe.clone()));
        drop(sub_manager);

        let pkid = un_subscribe.pkid;
        let mut user_properties = Vec::new();
        if let Some(properties) = un_subscribe_properties {
            user_properties = properties.user_properties;
        }
        return self.ack_build.unsub_ack(pkid, None, user_properties);
    }

    pub async fn disconnect(
        &self,
        connect_id: u64,
        disconnect: Disconnect,
        disconnect_properties: Option<DisconnectProperties>,
    ) -> MQTTPacket {
        let mut cache = self.metadata_cache.write().await;
        cache.remove_connect_id(connect_id);
        drop(cache);

        let mut heartbeat = self.heartbeat_manager.write().await;
        heartbeat.remove_connect(connect_id);
        drop(heartbeat);

        let mut sub_manager = self.subscribe_manager.write().await;
        sub_manager.remove_subscribe(connect_id, None);
        drop(sub_manager);

        return self
            .ack_build
            .distinct(DisconnectReasonCode::NormalDisconnection);
    }

    async fn authentication(&self, login: Option<Login>) -> bool {
        if login == None {
            return false;
        }
        let login_info = login.unwrap();
        let cache = self.metadata_cache.read().await;
        if let Some(user) = cache.user_info.get(&login_info.username) {
            return user.password == login_info.password;
        }
        return false;
    }
}
