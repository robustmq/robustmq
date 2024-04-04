use super::packet::MQTTAckBuild;
use crate::{
    metadata::{
        cache::MetadataCache,
        hearbeat::HeartbeatManager,
        session::{LastWillData, Session},
    },
    storage::storage::StorageLayer,
};
use common_base::{log::error, tools::unique_id_string};
use protocol::{
    mqtt::{
        Connect, ConnectProperties, Disconnect, DisconnectProperties, DisconnectReasonCode,
        LastWill, LastWillProperties, Login, MQTTPacket, PingReq, Publish, PublishProperties,
        Subscribe, SubscribeProperties, Unsubscribe, UnsubscribeProperties,
    },
    mqttv4::disconnect,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Mqtt5Service {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    storage_layer: StorageLayer,
    ack_build: MQTTAckBuild,
    heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
}

impl Mqtt5Service {
    pub fn new(
        metadata_cache: Arc<RwLock<MetadataCache>>,
        ack_build: MQTTAckBuild,
        heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
    ) -> Self {
        let storage_layer = StorageLayer::new();
        return Mqtt5Service {
            metadata_cache,
            ack_build,
            storage_layer,
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
                .storage_layer
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
            .storage_layer
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
        return self.ack_build.pub_ack();
    }

    pub fn subscribe(
        &self,
        subscribe: Subscribe,
        subscribe_properties: Option<SubscribeProperties>,
    ) -> MQTTPacket {
        return self.ack_build.sub_ack();
    }

    pub fn ping(&self, connect_id: u64, ping: PingReq) -> MQTTPacket {
        let mut heartbeat = self.heartbeat_manager.write().unwrap();
        heartbeat.report_hearbeat(connect_id);
        return self.ack_build.ping_resp();
    }

    pub fn un_subscribe(
        &self,
        un_subscribe: Unsubscribe,
        un_subscribe_properties: Option<UnsubscribeProperties>,
    ) -> MQTTPacket {
        return self.ack_build.unsub_ack();
    }

    pub fn disconnect(
        &self,
        disconnect: Disconnect,
        disconnect_properties: Option<DisconnectProperties>,
    ) -> MQTTPacket {
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
