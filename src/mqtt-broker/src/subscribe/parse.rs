use crate::{
    core::share_sub::{decode_share_info, is_share_sub},
    handler::subscribe::path_regex_match,
    metadata::subscriber::Subscriber,
    server::MQTTProtocol,
};
use clients::{placement::mqtt::call::placement_get_share_sub, poll::ClientPool};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError, log::info, tools::now_second,
};
use dashmap::DashMap;
use protocol::{
    mqtt::{Subscribe, SubscribeProperties},
    placement_center::generate::mqtt::GetShareSubRequest,
};
use std::sync::Arc;

pub async fn parse_subscribe(
    protocol: MQTTProtocol,
    client_id: String,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    client_poll: Arc<ClientPool>,
) -> Result<(), RobustMQError> {
    let sub_identifier = if let Some(properties) = subscribe_properties {
        properties.subscription_identifier
    } else {
        None
    };

    for (topic_id, topic_name) in self.metadata_cache.topic_id_name.clone() {
        if !self.topic_subscribe.contains_key(&topic_id) {
            self.topic_subscribe
                .insert(topic_id.clone(), DashMap::with_capacity(256));
        }

        if !self.client_subscribe.contains_key(&client_id) {
            self.client_subscribe
                .insert(client_id.clone(), DashMap::with_capacity(256));
        }

        let tp_sub = self.topic_subscribe.get_mut(&topic_id).unwrap();
        let client_sub = self.client_subscribe.get_mut(&client_id).unwrap();
        for filter in subscribe.filters.clone() {
            if is_share_sub(filter.path.clone()) {
                let (group_name, sub_name) = decode_share_info(filter.path.clone());
                if path_regex_match(topic_name.clone(), sub_name.clone()) {
                    let conf = broker_mqtt_conf();
                    let req = GetShareSubRequest {
                        cluster_name: conf.cluster_name.clone(),
                        group_name,
                        sub_name: sub_name.clone(),
                    };
                    match placement_get_share_sub(
                        client_poll.clone(),
                        conf.placement.server.clone(),
                        req,
                    )
                    .await
                    {
                        Ok(reply) => {
                            info(format!(
                                " Leader node for the shared subscription is [{}]",
                                reply.broker_id
                            ));
                            if reply.broker_id != conf.broker_id {
                                //todo
                            } else {
                                let sub = Subscriber {
                                    protocol: protocol.clone(),
                                    client_id: client_id.clone(),
                                    packet_identifier: subscribe.packet_identifier,
                                    qos: filter.qos,
                                    nolocal: filter.nolocal,
                                    preserve_retain: filter.preserve_retain,
                                    subscription_identifier: sub_identifier,
                                    user_properties: Vec::new(),
                                    is_share_sub: true,
                                };
                                tp_sub.insert(client_id.clone(), sub);
                                client_sub.insert(topic_id.clone(), now_second());
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            } else {
                if path_regex_match(topic_name.clone(), filter.path.clone()) {
                    let sub = Subscriber {
                        protocol: protocol.clone(),
                        client_id: client_id.clone(),
                        packet_identifier: subscribe.packet_identifier,
                        qos: filter.qos,
                        nolocal: filter.nolocal,
                        preserve_retain: filter.preserve_retain,
                        subscription_identifier: sub_identifier,
                        user_properties: Vec::new(),
                        is_share_sub: false,
                    };
                    tp_sub.insert(client_id.clone(), sub);
                    client_sub.insert(topic_id.clone(), now_second());
                }
            }
        }
    }
    return Ok(());
}
