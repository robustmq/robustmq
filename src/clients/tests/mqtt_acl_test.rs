mod common;
#[cfg(test)]
mod tests {
    use crate::common::get_placement_addr;
    use clients::{
        placement::mqtt::call::{create_acl, delete_acl, list_acl, list_blacklist},
        poll::ClientPool,
    };

    use metadata_struct::acl::mqtt_acl::{
        MQTTAcl, MQTTAclAction, MQTTAclPermission, MQTTAclResourceType,
    };
    use protocol::placement_center::generate::mqtt::{
        CreateAclRequest, DeleteAclRequest, ListAclRequest, ListBlacklistRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn mqtt_acl_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = "test1".to_string();

        let acl = MQTTAcl {
            resource_type: MQTTAclResourceType::User,
            resource_name: "loboxu".to_string(),
            topic: "tp-1".to_string(),
            ip: "*".to_string(),
            action: MQTTAclAction::All,
            permission: MQTTAclPermission::Deny,
        };

        let request = CreateAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };
        match create_acl(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = ListAclRequest {
            cluster_name: cluster_name.clone(),
        };

        match list_acl(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.acls {
                    let tmp = serde_json::from_slice::<MQTTAcl>(raw.as_slice()).unwrap();
                    if tmp.resource_type == acl.resource_type
                        && tmp.resource_name == acl.resource_name
                        && tmp.topic == acl.topic
                        && tmp.ip == acl.ip
                        && tmp.action == acl.action
                        && tmp.permission == acl.permission
                    {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = DeleteAclRequest {
            cluster_name: cluster_name.clone(),
            acl: acl.encode().unwrap(),
        };
        match delete_acl(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = ListBlacklistRequest {
            cluster_name: cluster_name.clone(),
        };

        match list_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.blacklists {
                    let tmp = serde_json::from_slice::<MQTTAcl>(raw.as_slice()).unwrap();
                    if tmp.resource_type == acl.resource_type
                        && tmp.resource_name == acl.resource_name
                        && tmp.topic == acl.topic
                        && tmp.ip == acl.ip
                        && tmp.action == acl.action
                        && tmp.permission == acl.permission
                    {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}
