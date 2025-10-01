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

use super::flow_control::is_qos_message;
use super::mqtt::{MqttService, MqttServiceConnectContext, MqttServiceContext};
use crate::handler::cache::MQTTCacheManager;
use crate::handler::connection::disconnect_connection;
use crate::handler::response::{
    response_packet_mqtt_connect_fail, response_packet_mqtt_distinct_by_reason,
};
use crate::security::AuthDriver;
use crate::subscribe::common::is_error_by_suback;
use crate::subscribe::manager::SubscribeManager;
use axum::async_trait;
use broker_core::cache::BrokerCacheManager;
use broker_core::rocksdb::RocksDBEngine;
use common_base::tools::now_mills;
use common_metrics::mqtt::event::{
    record_mqtt_connection_failed, record_mqtt_connection_success, record_mqtt_subscribe_failed,
    record_mqtt_subscribe_success, record_mqtt_unsubscribe_success,
};
use common_metrics::mqtt::time::record_mqtt_packet_process_duration;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::ResponsePackage;
use protocol::mqtt::common::{
    is_mqtt3, is_mqtt4, is_mqtt5, mqtt_packet_to_string, Connect, ConnectProperties,
    ConnectReturnCode, Disconnect, DisconnectProperties, DisconnectReasonCode, LastWill,
    LastWillProperties, Login, MqttPacket, MqttProtocol, PingReq, PubAck, PubAckProperties,
    PubComp, PubCompProperties, PubRec, PubRecProperties, PubRel, PubRelProperties, Publish,
    PublishProperties, Subscribe, SubscribeProperties, Unsubscribe, UnsubscribeProperties,
};
use protocol::robust::RobustMQPacket;
use schema_register::schema::SchemaRegisterManager;
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tracing::{debug, error};

// S: message storage adapter
#[derive(Clone)]
pub struct MQTTHandlerCommand {
    mqtt3_service: MqttService,
    mqtt4_service: MqttService,
    mqtt5_service: MqttService,
    cache_manager: Arc<MQTTCacheManager>,
    connection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
    pub client_pool: Arc<ClientPool>,
}

#[derive(Clone)]
pub struct CommandContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub message_storage_adapter: ArcStorageAdapter,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection_manager: Arc<ConnectionManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub auth_driver: Arc<AuthDriver>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub broker_cache: Arc<BrokerCacheManager>,
}

#[async_trait]
impl Command for MQTTHandlerCommand {
    async fn apply(
        &self,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        robust_packet: RobustMQPacket,
    ) -> Option<ResponsePackage> {
        let start = now_mills();
        let packet = robust_packet.get_mqtt_packet().unwrap();
        let mut is_connect_pkg = false;
        if let MqttPacket::Connect(_, _, _, _, _, _) = packet {
            is_connect_pkg = true;
        }

        if !is_connect_pkg && !self.check_login_status(tcp_connection.connection_id).await {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(response_packet_mqtt_distinct_by_reason(
                    &MqttProtocol::Mqtt4,
                    Some(DisconnectReasonCode::NotAuthorized),
                )),
            ));
        }

        let resp_package = match packet.clone() {
            MqttPacket::Connect(
                protocol_version,
                connect,
                properties,
                last_will,
                last_will_properties,
                login,
            ) => {
                self.process_connect(
                    &tcp_connection,
                    &addr,
                    protocol_version,
                    connect,
                    properties,
                    last_will,
                    last_will_properties,
                    login,
                )
                .await
            }

            MqttPacket::Publish(publish, publish_properties) => {
                self.process_publish(&tcp_connection, publish, publish_properties)
                    .await
            }

            MqttPacket::PubRec(pub_rec, pub_rec_properties) => {
                self.process_pubrec(&tcp_connection, &pub_rec, &pub_rec_properties)
                    .await
            }

            MqttPacket::PubComp(pub_comp, pub_comp_properties) => {
                self.process_pubcomp(&tcp_connection, &pub_comp, &pub_comp_properties)
                    .await
            }

            MqttPacket::PubRel(pub_rel, pub_rel_properties) => {
                self.process_pubrel(&tcp_connection, &pub_rel, &pub_rel_properties)
                    .await
            }

            MqttPacket::PubAck(pub_ack, pub_ack_properties) => {
                self.process_puback(&tcp_connection, &pub_ack, &pub_ack_properties)
                    .await
            }

            MqttPacket::Subscribe(subscribe, subscribe_properties) => {
                self.process_subscribe(&tcp_connection, &subscribe, &subscribe_properties)
                    .await
            }

            MqttPacket::PingReq(ping) => self.process_ping(&tcp_connection, &ping).await,

            MqttPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                self.process_unsubscribe(&tcp_connection, &unsubscribe, &unsubscribe_properties)
                    .await
            }

            MqttPacket::Disconnect(disconnect, disconnect_properties) => {
                self.process_disconnect(&tcp_connection, &disconnect, &disconnect_properties)
                    .await
            }

            _ => {
                return Some(ResponsePackage::build(
                    tcp_connection.connection_id,
                    RobustMQPacket::MQTT(response_packet_mqtt_connect_fail(
                        &MqttProtocol::Mqtt5,
                        ConnectReturnCode::MalformedPacket,
                        &None,
                        None,
                    )),
                ));
            }
        };

        if let Some(pkg) = resp_package.clone() {
            if let MqttPacket::Disconnect(_, _) = pkg.packet.get_mqtt_packet().unwrap() {
                if let Some(connection) = self
                    .cache_manager
                    .get_connection(tcp_connection.connection_id)
                {
                    if let Err(e) = disconnect_connection(
                        &connection.client_id,
                        connection.connect_id,
                        &self.cache_manager,
                        &self.client_pool,
                        &self.connection_manager,
                        &self.subscribe_manager,
                        true,
                    )
                    .await
                    {
                        error!("{}", e);
                    };
                }
            }
        }

        record_mqtt_packet_process_duration(
            tcp_connection.connection_type,
            mqtt_packet_to_string(&packet),
            (now_mills() - start) as f64,
        );
        resp_package
    }
}
impl MQTTHandlerCommand {
    #[allow(clippy::too_many_arguments)]
    pub async fn process_connect(
        &self,
        tcp_connection: &NetworkConnection,
        addr: &SocketAddr,
        protocol_version: u8,
        connect: Connect,
        properties: Option<ConnectProperties>,
        last_will: Option<LastWill>,
        last_will_properties: Option<LastWillProperties>,
        login: Option<Login>,
    ) -> Option<ResponsePackage> {
        self.connection_manager
            .set_mqtt_connect_protocol(tcp_connection.connection_id, protocol_version.to_owned());

        let resp_pkg = if is_mqtt3(protocol_version.to_owned()) {
            let connect_context = MqttServiceConnectContext {
                connect_id: tcp_connection.connection_id,
                connect: connect.clone(),
                connect_properties: properties.clone(),
                last_will: last_will.clone(),
                last_will_properties: last_will_properties.clone(),
                login: login.clone(),
                addr: *addr,
            };
            Some(self.mqtt3_service.connect(connect_context).await)
        } else if is_mqtt4(protocol_version.to_owned()) {
            let connect_context = MqttServiceConnectContext {
                connect_id: tcp_connection.connection_id,
                connect: connect.clone(),
                connect_properties: properties.clone(),
                last_will: last_will.clone(),
                last_will_properties: last_will_properties.clone(),
                login: login.clone(),
                addr: *addr,
            };
            Some(self.mqtt4_service.connect(connect_context).await)
        } else if is_mqtt5(protocol_version.to_owned()) {
            let connect_context = MqttServiceConnectContext {
                connect_id: tcp_connection.connection_id,
                connect: connect.clone(),
                connect_properties: properties.clone(),
                last_will: last_will.clone(),
                last_will_properties: last_will_properties.clone(),
                login: login.clone(),
                addr: *addr,
            };
            Some(self.mqtt5_service.connect(connect_context).await)
        } else {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(response_packet_mqtt_connect_fail(
                    &MqttProtocol::Mqtt5,
                    ConnectReturnCode::UnsupportedProtocolVersion,
                    &None,
                    None,
                )),
            ));
        };

        let ack_pkg = resp_pkg.unwrap();
        if let MqttPacket::ConnAck(conn_ack, _) = ack_pkg.clone() {
            if conn_ack.code == ConnectReturnCode::Success {
                let username = if let Some(user) = login {
                    user.username.to_owned()
                } else {
                    "".to_string()
                };
                self.cache_manager
                    .login_success(tcp_connection.connection_id, username);
                debug!("connect [{}] login success", tcp_connection.connection_id);
                record_mqtt_connection_success();
            } else {
                record_mqtt_connection_failed();
            }
        }
        Some(ResponsePackage::build(
            tcp_connection.connection_id,
            RobustMQPacket::MQTT(ack_pkg),
        ))
    }

    pub async fn process_publish(
        &self,
        tcp_connection: &NetworkConnection,
        publish: Publish,
        publish_properties: Option<PublishProperties>,
    ) -> Option<ResponsePackage> {
        let connection = if let Some(se) = self
            .cache_manager
            .connection_info
            .get(&tcp_connection.connection_id)
        {
            se.clone()
        } else {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(response_packet_mqtt_distinct_by_reason(
                    &tcp_connection.get_protocol(),
                    Some(DisconnectReasonCode::MaximumConnectTime),
                )),
            ));
        };

        if is_qos_message(publish.qos) {
            connection.recv_qos_message_incr();
        }

        let resp = if tcp_connection.is_mqtt3() {
            self.mqtt3_service
                .publish(tcp_connection.connection_id, &publish, &publish_properties)
                .await
        } else if tcp_connection.is_mqtt4() {
            self.mqtt4_service
                .publish(tcp_connection.connection_id, &publish, &publish_properties)
                .await
        } else if tcp_connection.is_mqtt5() {
            self.mqtt5_service
                .publish(tcp_connection.connection_id, &publish, &publish_properties)
                .await
        } else {
            None
        };

        if let Some(pack) = resp.clone() {
            if let MqttPacket::PubRec(_, _) = pack {
                // todo
            } else if is_qos_message(publish.qos) {
                connection.recv_qos_message_decr();
            }
        }
        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_pubrec(
        &self,
        tcp_connection: &NetworkConnection,
        pub_rec: &PubRec,
        pub_rec_properties: &Option<PubRecProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            self.mqtt3_service
                .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                .await
        } else if tcp_connection.is_mqtt4() {
            self.mqtt4_service
                .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                .await
        } else if tcp_connection.is_mqtt5() {
            self.mqtt5_service
                .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                .await
        } else {
            None
        };
        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_pubcomp(
        &self,
        tcp_connection: &NetworkConnection,
        pub_comp: &PubComp,
        pub_comp_properties: &Option<PubCompProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            self.mqtt3_service
                .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                .await
        } else if tcp_connection.is_mqtt4() {
            self.mqtt4_service
                .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                .await
        } else if tcp_connection.is_mqtt5() {
            self.mqtt5_service
                .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                .await
        } else {
            None
        };

        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_pubrel(
        &self,
        tcp_connection: &NetworkConnection,
        pub_rel: &PubRel,
        pub_rel_properties: &Option<PubRelProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            Some(
                self.mqtt3_service
                    .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                    .await,
            )
        } else if tcp_connection.is_mqtt4() {
            Some(
                self.mqtt4_service
                    .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                    .await,
            )
        } else if tcp_connection.is_mqtt5() {
            Some(
                self.mqtt5_service
                    .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                    .await,
            )
        } else {
            None
        };

        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_puback(
        &self,
        tcp_connection: &NetworkConnection,
        pub_ack: &PubAck,
        pub_ack_properties: &Option<PubAckProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            self.mqtt3_service
                .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                .await
        } else if tcp_connection.is_mqtt4() {
            self.mqtt4_service
                .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                .await
        } else if tcp_connection.is_mqtt5() {
            self.mqtt5_service
                .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                .await
        } else {
            None
        };
        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_subscribe(
        &self,
        tcp_connection: &NetworkConnection,
        subscribe: &Subscribe,
        subscribe_properties: &Option<SubscribeProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            Some(
                self.mqtt3_service
                    .subscribe(
                        tcp_connection.connection_id,
                        subscribe,
                        subscribe_properties,
                    )
                    .await,
            )
        } else if tcp_connection.is_mqtt4() {
            Some(
                self.mqtt4_service
                    .subscribe(
                        tcp_connection.connection_id,
                        subscribe,
                        subscribe_properties,
                    )
                    .await,
            )
        } else if tcp_connection.is_mqtt5() {
            Some(
                self.mqtt5_service
                    .subscribe(
                        tcp_connection.connection_id,
                        subscribe,
                        subscribe_properties,
                    )
                    .await,
            )
        } else {
            None
        };
        if let Some(pkg) = resp {
            if let MqttPacket::SubAck(sub_ack, _) = pkg.clone() {
                if is_error_by_suback(&sub_ack) {
                    record_mqtt_subscribe_failed();
                } else {
                    record_mqtt_subscribe_success();
                }
            }
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_ping(
        &self,
        tcp_connection: &NetworkConnection,
        ping: &PingReq,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            Some(
                self.mqtt3_service
                    .ping(tcp_connection.connection_id, ping)
                    .await,
            )
        } else if tcp_connection.is_mqtt4() {
            Some(
                self.mqtt4_service
                    .ping(tcp_connection.connection_id, ping)
                    .await,
            )
        } else if tcp_connection.is_mqtt5() {
            Some(
                self.mqtt5_service
                    .ping(tcp_connection.connection_id, ping)
                    .await,
            )
        } else {
            None
        };
        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_unsubscribe(
        &self,
        tcp_connection: &NetworkConnection,
        unsubscribe: &Unsubscribe,
        unsubscribe_properties: &Option<UnsubscribeProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            Some(
                self.mqtt3_service
                    .un_subscribe(
                        tcp_connection.connection_id,
                        unsubscribe,
                        unsubscribe_properties,
                    )
                    .await,
            )
        } else if tcp_connection.is_mqtt4() {
            Some(
                self.mqtt4_service
                    .un_subscribe(
                        tcp_connection.connection_id,
                        unsubscribe,
                        unsubscribe_properties,
                    )
                    .await,
            )
        } else if tcp_connection.is_mqtt5() {
            Some(
                self.mqtt5_service
                    .un_subscribe(
                        tcp_connection.connection_id,
                        unsubscribe,
                        unsubscribe_properties,
                    )
                    .await,
            )
        } else {
            None
        };

        if let Some(pkg) = resp {
            record_mqtt_unsubscribe_success();
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }

    pub async fn process_disconnect(
        &self,
        tcp_connection: &NetworkConnection,
        disconnect: &Disconnect,
        disconnect_properties: &Option<DisconnectProperties>,
    ) -> Option<ResponsePackage> {
        let resp = if tcp_connection.is_mqtt3() {
            self.mqtt3_service
                .disconnect(
                    tcp_connection.connection_id,
                    disconnect,
                    disconnect_properties,
                )
                .await
        } else if tcp_connection.is_mqtt4() {
            self.mqtt4_service
                .disconnect(
                    tcp_connection.connection_id,
                    disconnect,
                    disconnect_properties,
                )
                .await
        } else if tcp_connection.is_mqtt5() {
            self.mqtt5_service
                .disconnect(
                    tcp_connection.connection_id,
                    disconnect,
                    disconnect_properties,
                )
                .await
        } else {
            None
        };

        if let Some(pkg) = resp {
            return Some(ResponsePackage::build(
                tcp_connection.connection_id,
                RobustMQPacket::MQTT(pkg),
            ));
        }
        None
    }
}

impl MQTTHandlerCommand {
    pub fn new(context: CommandContext) -> Self {
        let mqtt3_context = MqttServiceContext {
            protocol: MqttProtocol::Mqtt3,
            cache_manager: context.cache_manager.clone(),
            connection_manager: context.connection_manager.clone(),
            message_storage_adapter: context.message_storage_adapter.clone(),
            delay_message_manager: context.delay_message_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            schema_manager: context.schema_manager.clone(),
            client_pool: context.client_pool.clone(),
            auth_driver: context.auth_driver.clone(),
            rocksdb_engine_handler: context.rocksdb_engine_handler.clone(),
        };
        let mqtt3_service = MqttService::new(mqtt3_context);
        let mqtt4_context = MqttServiceContext {
            protocol: MqttProtocol::Mqtt4,
            cache_manager: context.cache_manager.clone(),
            connection_manager: context.connection_manager.clone(),
            message_storage_adapter: context.message_storage_adapter.clone(),
            delay_message_manager: context.delay_message_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            schema_manager: context.schema_manager.clone(),
            client_pool: context.client_pool.clone(),
            auth_driver: context.auth_driver.clone(),
            rocksdb_engine_handler: context.rocksdb_engine_handler.clone(),
        };
        let mqtt4_service = MqttService::new(mqtt4_context);
        let mqtt5_context = MqttServiceContext {
            protocol: MqttProtocol::Mqtt5,
            cache_manager: context.cache_manager.clone(),
            connection_manager: context.connection_manager.clone(),
            message_storage_adapter: context.message_storage_adapter.clone(),
            delay_message_manager: context.delay_message_manager.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            schema_manager: context.schema_manager.clone(),
            client_pool: context.client_pool.clone(),
            auth_driver: context.auth_driver.clone(),
            rocksdb_engine_handler: context.rocksdb_engine_handler.clone(),
        };
        let mqtt5_service = MqttService::new(mqtt5_context);
        MQTTHandlerCommand {
            mqtt3_service,
            mqtt4_service,
            mqtt5_service,
            client_pool: context.client_pool.clone(),
            subscribe_manager: context.subscribe_manager.clone(),
            cache_manager: context.cache_manager,
            connection_manager: context.connection_manager,
        }
    }

    pub async fn check_login_status(&self, connection_id: u64) -> bool {
        self.cache_manager.is_login(connection_id)
    }
}

pub fn create_command(command_context: CommandContext) -> Arc<Box<dyn Command + Send + Sync>> {
    let storage: Box<dyn Command + Send + Sync> =
        Box::new(MQTTHandlerCommand::new(command_context));
    Arc::new(storage)
}
