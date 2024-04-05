use super::packet::MQTTAckBuild;
use crate::{
    metadata::{
        cache::MetadataCache,
        hearbeat::HeartbeatManager,
        message::Message,
        session::{LastWillData, Session},
        subscriber::Subscriber,
        topic::Topic,
    },
    storage::{message::MessageStorage, metadata::MetadataStorage},
    subscribe::subscribe_manager::SubScribeManager,
};
use common_base::{log::error, tools::unique_id_string};
use protocol::mqtt::{
    Connect, ConnectProperties, Disconnect, DisconnectProperties, DisconnectReasonCode, LastWill,
    LastWillProperties, Login, MQTTPacket, PingReq, PubAck, PubAckProperties, Publish,
    PublishProperties, Subscribe, SubscribeProperties, Unsubscribe, UnsubscribeProperties,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Mqtt5Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    subscribe_manager: Arc<RwLock<SubScribeManager>>,
    metadata_storage: MetadataStorage,
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
        let metadata_storage = MetadataStorage::new();
        let message_storage = MessageStorage::new();
        return Mqtt5Service {
            metadata_cache,
            subscribe_manager,
            message_storage,
            ack_build,
            metadata_storage,
            heartbeat_manager,
        };
    }

    pub fn connect(
        &mut self,
        connect_id: u64,
        connnect: Connect,
        connect_properties: Option<ConnectProperties>,
        last_will: Option<LastWill>,
        last_will_properties: Option<LastWillProperties>,
        login: Option<Login>,
    ) -> MQTTPacket {
        // connect for authentication
        if self.authentication(login) {
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

        session = session.build_session(
            client_id.clone(),
            connnect.clone(),
            connect_properties.clone(),
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
        match self
            .metadata_storage
            .save_session(client_id.clone(), session.clone())
        {
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
        let mut cache = self.metadata_cache.write().unwrap();
        cache.set_session(client_id.clone(), session);
        cache.set_client_id(connect_id, client_id.clone());
        drop(cache);
        let mut heartbeat = self.heartbeat_manager.write().unwrap();
        heartbeat.report_hearbeat(connect_id);
        drop(heartbeat);

        let mut user_properties = Vec::new();
        if let Some(connect_properties) = connect_properties {
            user_properties = connect_properties.user_properties;
        }

        let reason_string = Some("".to_string());
        let response_information = Some("".to_string());
        let server_reference = Some("".to_string());
        return self.ack_build.conn_ack(
            client_id,
            auto_client_id,
            reason_string,
            user_properties,
            response_information,
            server_reference,
        );
    }

    pub fn publish(
        &self,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> MQTTPacket {
        let topic_name = String::from_utf8(publish.topic.to_vec()).unwrap();
        let topic = Topic::new(&topic_name);

        // Update the cache if the Topic doesn't exist and persist the Topic information
        let mut cache = self.metadata_cache.write().unwrap();
        if !cache.topic_exists(&topic_name) {
            cache.set_topic(&topic_name, &topic);
            match self.metadata_storage.save_topic(&topic_name, &topic) {
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

    pub fn publish_ack(&self, pub_ack: PubAck, puback_properties: Option<PubAckProperties>) {

    }
    
    pub fn subscribe(
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
        let mut sub_manager = self.subscribe_manager.write().unwrap();
        sub_manager.add_subscribe(connect_id, subscriber);
        drop(sub_manager);

        let pkid = subscribe.packet_identifier;
        let mut user_properties = Vec::new();
        if let Some(properties) = subscribe_properties {
            user_properties = properties.user_properties;
        }
        return self.ack_build.sub_ack(pkid, None, user_properties);
    }

    pub fn ping(&self, connect_id: u64, _: PingReq) -> MQTTPacket {
        let mut heartbeat = self.heartbeat_manager.write().unwrap();
        heartbeat.report_hearbeat(connect_id);
        return self.ack_build.ping_resp();
    }

    pub fn un_subscribe(
        &self,
        connect_id: u64,
        un_subscribe: Unsubscribe,
        un_subscribe_properties: Option<UnsubscribeProperties>,
    ) -> MQTTPacket {
        // Remove subscription information
        let mut sub_manager = self.subscribe_manager.write().unwrap();
        sub_manager.remove_subscribe(connect_id, Some(un_subscribe.clone()));
        drop(sub_manager);

        let pkid = un_subscribe.pkid;
        let mut user_properties = Vec::new();
        if let Some(properties) = un_subscribe_properties {
            user_properties = properties.user_properties;
        }
        return self.ack_build.unsub_ack(pkid, None, user_properties);
    }

    pub fn disconnect(
        &self,
        connect_id: u64,
        disconnect: Disconnect,
        disconnect_properties: Option<DisconnectProperties>,
    ) -> MQTTPacket {
        let mut cache = self.metadata_cache.write().unwrap();
        cache.remove_connect_id(connect_id);
        drop(cache);

        let mut heartbeat = self.heartbeat_manager.write().unwrap();
        heartbeat.remove_connect(connect_id);
        drop(heartbeat);

        let mut sub_manager = self.subscribe_manager.write().unwrap();
        sub_manager.remove_subscribe(connect_id, None);
        drop(sub_manager);

        return self
            .ack_build
            .distinct(DisconnectReasonCode::NormalDisconnection);
    }

    fn authentication(&self, login: Option<Login>) -> bool {
        if login == None {
            return false;
        }
        let login_info = login.unwrap();
        let cache = self.metadata_cache.read().unwrap();
        if let Some(user) = cache.user_info.get(&login_info.username) {
            return user.password == login_info.password;
        }
        return false;
    }
}
