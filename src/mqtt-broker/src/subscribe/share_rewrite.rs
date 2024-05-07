use common_base::log::error;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use protocol::{
    mqtt::{MQTTPacket, PublishProperties, Subscribe, SubscribeProperties},
    mqttv4::codec::Mqtt4Codec,
};
use std::time::Duration;
use tokio::{net::TcpStream, sync::broadcast, time::sleep};
use tokio_util::codec::Framed;

#[derive(Clone)]
pub struct ShareSubRewriteSub {
    pub client_id: String,
    pub group_name: String,
    pub sub_name: String,
    pub leader_id: u64,
    pub leader_addr: String,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
}

#[derive(Clone)]
pub struct SubscribeShare {
    pub share_sub_list: DashMap<String, ShareSubRewriteSub>,
    pub share_sub_thread: DashMap<String, String>,
}

impl SubscribeShare {
    pub fn new() -> Self {
        return SubscribeShare {
            share_sub_list: DashMap::with_capacity(128),
            share_sub_thread: DashMap::with_capacity(128),
        };
    }

    pub fn start() {}

    pub fn add_share_sub_rewrite_sub(&self, client_id: String, rewrite_sub: ShareSubRewriteSub) {
        self.share_sub_list.insert(client_id, rewrite_sub);
    }

    pub fn remove_share_sub_rewrite_sub(&self, client_id: String) {
        self.share_sub_list.remove(&client_id);
    }

    pub async fn start_rewrite_sub(&self) {
        loop {
            for (client_id, re_sub) in self.share_sub_list.clone() {
                if !self.share_sub_thread.contains_key(&client_id) {
                    let (stop_send, _) = broadcast::channel::<bool>(2);
                    tokio::spawn(async move {
                        let socket = TcpStream::connect(re_sub.leader_addr.clone())
                            .await
                            .unwrap();
                        let mut stream: Framed<TcpStream, Mqtt4Codec> =
                            Framed::new(socket, Mqtt4Codec::new());

                        // send connect package
                        let packet = build_rewrite_subscribe_pkg(re_sub);
                        let _ = stream.send(packet).await;
                        let mut recv = stop_send.subscribe();
                        loop {
                            if let Some(data) = stream.next().await {
                                match data {
                                    Ok(da) => match da {
                                        MQTTPacket::Publish(publish, publish_properties) => {}

                                        MQTTPacket::PubRec(publish, publish_properties) => {}

                                        MQTTPacket::PubComp(publish, publish_properties) => {}

                                        MQTTPacket::Disconnect(publish, publish_properties) => {}
                                        _ => {
                                            error("".to_string());
                                        }
                                    },
                                    Err(e) => error(e.to_string()),
                                }
                            }
                            match recv.try_recv() {
                                Ok(flag) => {
                                    if flag {
                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    });
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    // Handles message push tasks for shared subscriptions
    // If the current Broker is the Leader node of the subscription, the message is pushed directly to the client.
    // If the current Broker is not the subscribed Leader node, it initiates a subscription task to the subscribed Leader node,
    // receives the data pushed by the Leader node, and returns it to the client.
    pub fn share_sub_action_thread(&self) {}
}

const SHARE_SUB_PREFIX: &str = "$share";
const SHARE_SUB_REWRITE_PUBLISH_FLAG: &str = "$system_ssrpf";
const SHARE_SUB_REWRITE_PUBLISH_FLAG_VALUE: &str = "True";

pub fn build_rewrite_subscribe_pkg(rewrite_sub: ShareSubRewriteSub) -> MQTTPacket {
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
    use crate::subscribe::share_rewrite::is_share_sub;

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
