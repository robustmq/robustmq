use super::manager::SubscribeManager;
use crate::{
    core::metadata_cache::MetadataCacheManager, metadata::subscriber::Subscriber,
    server::tcp::packet::ResponsePackage, storage::message::MessageStorage,
    subscribe::manager::ShareSubShareSub,
};
use clients::poll::ClientPool;
use common_base::log::{error, info};
use dashmap::DashMap;
use protocol::mqtt::{MQTTPacket, PublishProperties, SubscribeProperties};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    sync::broadcast::{self, Sender},
    time::sleep,
};

const SHARE_SUB_PREFIX: &str = "$share";
const SHARE_SUB_REWRITE_PUBLISH_FLAG: &str = "$system_ssrpf";
const SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE: &str = "True";

#[derive(Clone)]
pub struct SubscribeShare {
    // (topic_id, Sender<bool>)
    pub leader_push_thread: DashMap<String, Sender<bool>>,

    // (topic_id, Sender<bool>)
    pub follower_sub_thread: DashMap<String, Sender<bool>>,

    pub subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
}

impl SubscribeShare {
    pub fn new(client_poll: Arc<ClientPool>, subscribe_manager: Arc<SubscribeManager>) -> Self {
        return SubscribeShare {
            leader_push_thread: DashMap::with_capacity(128),
            follower_sub_thread: DashMap::with_capacity(128),
            subscribe_manager,
            client_poll,
        };
    }

    pub async fn start_leader_push(&self) {
        loop {
            for (topic_id, sub_list) in self.subscribe_manager.share_leader_subscribe.clone() {
                if sub_list.len() == 0 {
                    if let Some(sx) = self.leader_push_thread.get(&topic_id) {
                        match sx.send(true) {
                            Ok(_) => {
                                self.leader_push_thread.remove(&topic_id);
                            }
                            Err(e) => error(e.to_string()),
                        }
                    }
                    self.subscribe_manager
                        .share_leader_subscribe
                        .remove(&topic_id);
                }

                if self.leader_push_thread.contains_key(&topic_id) {
                    continue;
                }

                let (sx, mut rx) = broadcast::channel(2);
                tokio::spawn(async move {
                    info(format!(
                        "Share push thread for Topic [{}] was started successfully",
                        topic_id
                    ));
                    loop {
                        match rx.try_recv() {
                            Ok(flag) => {
                                if flag {
                                    info(format!(
                                        "Exclusive Push thread for Topic [{}] was stopped successfully",
                                        topic_id
                                    ));
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                        if let Some(sub_list) =
                            self.subscribe_manager.share_leader_subscribe.get(&topic_id)
                        {
                            push_thread(
                                sub_list.clone(),
                                metadata_cache.clone(),
                                message_storage.clone(),
                                topic_id.clone(),
                                response_queue_sx4.clone(),
                                response_queue_sx5.clone(),
                            )
                            .await;
                        } else {
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                });
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn start_follower_sub(&self) {
        // tokio::spawn(async move {
        //     let socket = TcpStream::connect(re_sub.leader_addr.clone())
        //         .await
        //         .unwrap();
        //     let mut stream: Framed<TcpStream, Mqtt4Codec> =
        //         Framed::new(socket, Mqtt4Codec::new());

        //     // send connect package
        //     let packet = build_rewrite_subscribe_pkg(re_sub);
        //     let _ = stream.send(packet).await;
        //     let mut recv = stop_send.subscribe();
        //     loop {
        //         if let Some(data) = stream.next().await {
        //             match data {
        //                 Ok(da) => match da {
        //                     MQTTPacket::Publish(publish, publish_properties) => {}

        //                     MQTTPacket::PubRec(publish, publish_properties) => {}

        //                     MQTTPacket::PubComp(publish, publish_properties) => {}

        //                     MQTTPacket::Disconnect(publish, publish_properties) => {}
        //                     _ => {
        //                         error("".to_string());
        //                     }
        //                 },
        //                 Err(e) => error(e.to_string()),
        //             }
        //         }
        //         match recv.try_recv() {
        //             Ok(flag) => {
        //                 if flag {
        //                     break;
        //                 }
        //             }
        //             Err(_) => {}
        //         }
        //     }
        // });
    }
    // Handles message push tasks for shared subscriptions
    // If the current Broker is the Leader node of the subscription, the message is pushed directly to the client.
    // If the current Broker is not the subscribed Leader node, it initiates a subscription task to the subscribed Leader node,
    // receives the data pushed by the Leader node, and returns it to the client.
    pub fn share_sub_action_thread(&self) {}
}

async fn leader_push_thread<S>(
    sub_list: DashMap<String, Subscriber>,
    metadata_cache: Arc<MetadataCacheManager>,
    message_storage: Arc<S>,
    topic_id: String,
    response_queue_sx4: Sender<ResponsePackage>,
    response_queue_sx5: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let message_storage = MessageStorage::new(message_storage);
    let group_id = format!("system_sub_{}", topic_id);
    let record_num = sub_list.len() * 3;
    let max_wait_ms = 500;

    match message_storage
        .read_topic_message(topic_id.clone(), group_id.clone(), record_num as u128)
        .await
    {
        Ok(results) => {
            if results.len() == 0 {
                sleep(Duration::from_millis(max_wait_ms)).await;
                return;
            }

            // commit offset
            if let Some(last_res) = results.last() {
                match message_storage
                    .commit_group_offset(topic_id.clone(), group_id.clone(), last_res.offset)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        error(e.to_string());
                        return;
                    }
                }
            }

            // Push data to subscribers
            for record in results.clone() {
                
            }
        }
        Err(e) => {
            error(e.to_string());
            sleep(Duration::from_millis(max_wait_ms)).await;
        }
    }
}

fn push_by_round_robin() {
    let mut sub_id = Vec::new();
    if let Some(id) = subscribe.subscription_identifier {
        sub_id.push(id);
    }

    let connect_id =
        if let Some(sess) = metadata_cache.session_info.get(&subscribe.client_id) {
            if let Some(conn_id) = sess.connection_id {
                conn_id
            } else {
                continue;
            }
        } else {
            continue;
        };
    let msg = match Message::decode_record(record) {
        Ok(msg) => msg,
        Err(e) => {
            error(e.to_string());
            continue;
        }
    };
    let publish = Publish {
        dup: false,
        qos: max_qos(msg.qos, subscribe.qos),
        pkid: subscribe.packet_identifier,
        retain: false,
        topic: Bytes::from(topic_name.clone()),
        payload: Bytes::from(msg.payload),
    };

    // If it is a shared subscription, it will be identified with the push message
    let mut user_properteis = Vec::new();
    if subscribe.is_share_sub {
        user_properteis.push(share_sub_rewrite_publish_flag());
    }

    let properties = PublishProperties {
        payload_format_indicator: None,
        message_expiry_interval: None,
        topic_alias: None,
        response_topic: None,
        correlation_data: None,
        user_properties: user_properteis,
        subscription_identifiers: sub_id.clone(),
        content_type: None,
    };

    let resp = ResponsePackage {
        connection_id: connect_id,
        packet: MQTTPacket::Publish(publish, Some(properties)),
    };

    if subscribe.protocol == MQTTProtocol::MQTT4 {
        match response_queue_sx4.send(resp) {
            Ok(_) => {}
            Err(e) => error(format!("{}", e.to_string())),
        }
    } else if subscribe.protocol == MQTTProtocol::MQTT5 {
        match response_queue_sx5.send(resp) {
            Ok(_) => {}
            Err(e) => error(format!("{}", e.to_string())),
        }
    }
}

fn push_by_random() {}

fn push_by_hash() {}

fn push_by_sticky() {}

fn push_by_local() {}

pub fn build_rewrite_subscribe_pkg(rewrite_sub: ShareSubShareSub) -> MQTTPacket {
    let subscribe = rewrite_sub.subscribe.clone();
    let mut subscribe_properties =
        if let Some(properties) = rewrite_sub.subscribe_properties.clone() {
            properties
        } else {
            SubscribeProperties::default()
        };

    let mut user_properties = Vec::new();
    user_properties.push(share_sub_rewrite_publish_flag());
    subscribe_properties.user_properties = user_properties;
    return MQTTPacket::Subscribe(subscribe, Some(subscribe_properties));
}

pub fn is_share_sub(sub_name: String) -> bool {
    return sub_name.starts_with(SHARE_SUB_PREFIX);
}

pub fn decode_share_info(sub_name: String) -> (String, String) {
    let mut str_slice: Vec<&str> = sub_name.split("/").collect();
    str_slice.remove(0);
    let group_name = str_slice.remove(0).to_string();
    let sub_name = format!("/{}", str_slice.join("/"));
    return (group_name, sub_name);
}

pub fn is_share_sub_rewrite_publish(publish_properties: Option<PublishProperties>) -> bool {
    if let Some(properties) = publish_properties {
        for (k, v) in properties.user_properties {
            if k == SHARE_SUB_REWRITE_PUBLISH_FLAG
                && v == SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE.to_string()
            {
                return true;
            }
        }
    }
    return false;
}

pub fn share_sub_rewrite_publish_flag() -> (String, String) {
    return (
        SHARE_SUB_REWRITE_PUBLISH_FLAG.to_string(),
        SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE.to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::decode_share_info;
    use crate::subscribe::share_sub::is_share_sub;

    #[tokio::test]
    async fn is_share_sub_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        assert!(is_share_sub(sub1));
        assert!(is_share_sub(sub2));
        assert!(is_share_sub(sub3));
        assert!(is_share_sub(sub4));

        let sub5 = "/comsumer1/$share/finance/#".to_string();
        let sub6 = "/comsumer1/$share/finance/$share".to_string();

        assert!(!is_share_sub(sub5));
        assert!(!is_share_sub(sub6));
    }

    #[tokio::test]
    async fn decode_share_info_test() {
        let sub1 = "$share/consumer1/sport/tennis/+".to_string();
        let sub2 = "$share/consumer2/sport/tennis/+".to_string();
        let sub3 = "$share/consumer1/sport/#".to_string();
        let sub4 = "$share/comsumer1/finance/#".to_string();

        let (group_name, topic_name) = decode_share_info(sub1);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(sub2);
        assert_eq!(group_name, "consumer2".to_string());
        assert_eq!(topic_name, "/sport/tennis/+".to_string());

        let (group_name, topic_name) = decode_share_info(sub3);
        assert_eq!(group_name, "consumer1".to_string());
        assert_eq!(topic_name, "/sport/#".to_string());

        let (group_name, topic_name) = decode_share_info(sub4);
        assert_eq!(group_name, "comsumer1".to_string());
        assert_eq!(topic_name, "/finance/#".to_string());
    }
}
