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

#[cfg(test)]
mod tests {
    use crate::mqtt::protocol::common::{
        broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
        ssl_by_type, ws_by_type,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::blacklist::{
        BlackListListReq, BlackListListRow, CreateBlackListReq, DeleteBlackListReq,
    };
    use admin_server::mqtt::user::{CreateUserReq, DeleteUserReq};
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use metadata_struct::auth::blacklist::EnumBlackListType as MqttAclBlackListType;
    use paho_mqtt::MessageBuilder;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn blacklist_storage_test() {
        let admin_client = create_test_env().await;
        let user = unique_id();
        let password: String = unique_id();
        let blacklist_name = format!("bl-storage-test-{}", unique_id());

        // create
        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::User.to_string(),
            resource_name: user.clone(),
            end_time: now_second() + 10000,
            desc: Some(password.clone()),
        };

        create_user(&admin_client, &user, &password).await;
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();
        sleep(Duration::from_secs(3)).await;

        // list
        let list_request = BlackListListReq {
            tenant: None,
            name: Some(blacklist_name.clone()),
            resource_name: None,
            limit: Some(10000),
            page: Some(1),
            sort_field: None,
            sort_by: None,
        };
        let page_data = admin_client
            .get_blacklist::<BlackListListReq, Vec<BlackListListRow>>(&list_request)
            .await
            .unwrap();
        let found = page_data.data.iter().any(|r| r.name == blacklist_name);
        assert!(found);

        // delete
        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();
        sleep(Duration::from_secs(3)).await;

        // list again
        let page_data = admin_client
            .get_blacklist::<BlackListListReq, Vec<BlackListListRow>>(&list_request)
            .await
            .unwrap();
        let found = page_data.data.iter().any(|r| r.name == blacklist_name);
        assert!(!found);

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    async fn blacklist_user_auth_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let blacklist_name = format!("bl-user-auth-{}", unique_id());

        create_user(&admin_client, &user, &password).await;

        // create blacklist
        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::User.to_string(),
            resource_name: user.clone(),
            end_time: now_second() + 10000,
            desc: None,
        };
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();

        let topic = unique_id();

        // push deny true
        publish_deny_test(&topic, &user, &password, true).await;

        // delete blacklist
        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();

        // push deny false
        publish_deny_test(&topic, &user, &password, false).await;
        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    async fn blacklist_client_id_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let client_id = format!("blacklist_client_{}", unique_id());
        let blacklist_name = format!("bl-client-id-{}", unique_id());

        create_user(&admin_client, &user, &password).await;

        // create blacklist for client_id
        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::ClientId.to_string(),
            resource_name: client_id.clone(),
            end_time: now_second() + 10000,
            desc: Some("ClientId blacklist test".to_string()),
        };
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();

        let topic = unique_id();

        // publish deny true
        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, true).await;

        // delete blacklist
        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();

        // publish deny false
        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    #[ignore = "127.0.0.1 whitelist affects other test cases"]
    async fn blacklist_ip_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let blacklist_name = format!("bl-ip-{}", unique_id());

        create_user(&admin_client, &user, &password).await;

        // create blacklist for IP
        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::Ip.to_string(),
            resource_name: "127.0.0.1".to_string(),
            end_time: now_second() + 10000,
            desc: Some("IP blacklist test".to_string()),
        };
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();

        let topic = unique_id();

        publish_deny_test(&topic, &user, &password, true).await;

        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();

        publish_deny_test(&topic, &user, &password, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    async fn blacklist_client_id_match_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let client_id_prefix = format!("test_client_{}", unique_id());
        let client_id = format!("{}_001", client_id_prefix);
        let blacklist_name = format!("bl-client-id-match-{}", unique_id());

        create_user(&admin_client, &user, &password).await;

        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::ClientIdMatch.to_string(),
            resource_name: format!("{}*", client_id_prefix),
            end_time: now_second() + 10000,
            desc: Some("ClientId pattern match test".to_string()),
        };
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();

        let topic = unique_id();

        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, true).await;

        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();

        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    async fn blacklist_user_match_test() {
        let admin_client = create_test_env().await;

        let user_prefix = format!("test_user_{}", unique_id());
        let user = format!("{}_001", user_prefix);
        let password: String = unique_id();
        let blacklist_name = format!("bl-user-match-{}", unique_id());

        create_user(&admin_client, &user, &password).await;

        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::UserMatch.to_string(),
            resource_name: format!("{}*", user_prefix),
            end_time: now_second() + 10000,
            desc: Some("User pattern match test".to_string()),
        };
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();

        let topic = unique_id();

        publish_deny_test(&topic, &user, &password, true).await;

        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();

        publish_deny_test(&topic, &user, &password, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    #[ignore = "127.0.0.1 whitelist affects other test cases"]
    async fn blacklist_ip_cidr_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let blacklist_name = format!("bl-ip-cidr-{}", unique_id());

        create_user(&admin_client, &user, &password).await;

        let create_request = CreateBlackListReq {
            name: blacklist_name.clone(),
            tenant: "default".to_string(),
            blacklist_type: MqttAclBlackListType::IPCIDR.to_string(),
            resource_name: "127.0.0.0/24".to_string(),
            end_time: now_second() + 10000,
            desc: Some("IP CIDR blacklist test".to_string()),
        };
        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();

        let topic = unique_id();

        publish_deny_test(&topic, &user, &password, true).await;

        let delete_request = DeleteBlackListReq {
            tenant: "default".to_string(),
            name: blacklist_name.clone(),
        };
        admin_client
            .delete_blacklist(&delete_request)
            .await
            .unwrap();

        publish_deny_test(&topic, &user, &password, false).await;

        delete_user(&admin_client, &user).await;
    }

    async fn publish_deny_test(topic: &str, username: &str, password: &str, is_err: bool) {
        let protocol = 5;
        let network = "tcp";
        let qos = 1;
        let client_id =
            build_client_id(format!("publish_deny_test_{protocol}_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            user_name: username.to_owned(),
            password: password.to_owned(),
            conn_is_err: is_err,
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let message = "publish_deny_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.to_owned())
            .qos(qos)
            .retained(true)
            .finalize();

        let result = cli.publish(msg);

        if result.is_err() {
            println!("{client_id},{result:?},{is_err}");
        }

        if is_err {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }

        distinct_conn(cli);
    }

    async fn publish_deny_test_with_client_id(
        topic: &str,
        username: &str,
        password: &str,
        client_id: &str,
        is_err: bool,
    ) {
        let protocol = 5;
        let network = "tcp";
        let qos = 1;

        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ws: ws_by_type(network),
            ssl: ssl_by_type(network),
            user_name: username.to_owned(),
            password: password.to_owned(),
            conn_is_err: is_err,
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        let message = "publish_deny_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.to_owned())
            .qos(qos)
            .retained(true)
            .finalize();

        let result = cli.publish(msg);

        if result.is_err() {
            println!("{client_id},{result:?},{is_err}");
        }

        if is_err {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }

        distinct_conn(cli);
    }

    async fn create_user(admin_client: &AdminHttpClient, username: &str, password: &str) {
        let user = CreateUserReq {
            tenant: "default".to_string(),
            username: username.to_owned(),
            password: password.to_owned(),
            is_superuser: false,
        };
        let res = admin_client.create_user(&user).await;
        assert!(res.is_ok());
    }

    async fn delete_user(admin_client: &AdminHttpClient, username: &str) {
        let user = DeleteUserReq {
            tenant: "default".to_string(),
            username: username.to_owned(),
        };
        let res = admin_client.delete_user(&user).await;
        assert!(res.is_ok());
    }
}
