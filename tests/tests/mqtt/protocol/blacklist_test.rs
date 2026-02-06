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
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
    use paho_mqtt::MessageBuilder;

    #[tokio::test]
    async fn blacklist_storage_test() {
        let admin_client = create_test_env().await;
        let user = unique_id();
        let password: String = unique_id();

        // create
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: user.to_string(),
            end_time: 10000,
            desc: password.to_string(),
        };

        create_blacklist(&admin_client, blacklist.clone()).await;

        // list
        let list_request = BlackListListReq {
            limit: Some(10000),
            page: Some(1),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };
        let mut flag = false;
        let page_data = admin_client
            .get_blacklist::<BlackListListReq, Vec<BlackListListRow>>(&list_request)
            .await
            .unwrap();
        for raw in page_data.data {
            if raw.resource_name == blacklist.resource_name {
                flag = true;
                break;
            }
        }
        assert!(flag);

        // delete
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // list
        let mut flag = false;
        let page_data = admin_client
            .get_blacklist::<BlackListListReq, Vec<BlackListListRow>>(&list_request)
            .await
            .unwrap();
        for raw in page_data.data {
            if raw.resource_name == blacklist.resource_name {
                flag = true;
                break;
            }
        }
        assert!(!flag);
    }

    #[tokio::test]
    async fn blacklist_user_auth_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();

        // create blacklist
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: user.to_string(),
            end_time: now_second() + 10000,
            desc: password.to_string(),
        };

        create_blacklist(&admin_client, blacklist.clone()).await;

        let topic = unique_id();

        // push deny true
        publish_deny_test(&topic, &user, &password, true).await;

        // delete blacklist
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // push deny false
        publish_deny_test(&topic, &user, &password, false).await;
        delete_user(&admin_client, &blacklist.resource_name).await;
    }

    #[tokio::test]
    async fn blacklist_client_id_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let client_id = format!("blacklist_client_{}", unique_id());

        // create user first
        create_user(&admin_client, &user, &password).await;

        // create blacklist for client_id
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientId,
            resource_name: client_id.clone(),
            end_time: now_second() + 10000,
            desc: "ClientId blacklist test".to_string(),
        };

        create_blacklist_without_user(&admin_client, blacklist.clone()).await;

        let topic = unique_id();

        // publish deny true - should fail with blacklisted client_id
        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, true).await;

        // delete blacklist
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // publish deny false - should succeed after blacklist removed
        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn blacklist_ip_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();

        // create user first
        create_user(&admin_client, &user, &password).await;

        // create blacklist for IP (localhost/127.0.0.1)
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::Ip,
            resource_name: "127.0.0.1".to_string(),
            end_time: now_second() + 10000,
            desc: "IP blacklist test".to_string(),
        };

        create_blacklist_without_user(&admin_client, blacklist.clone()).await;

        let topic = unique_id();

        // publish deny true - should fail from blacklisted IP
        publish_deny_test(&topic, &user, &password, true).await;

        // delete blacklist
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // publish deny false - should succeed after blacklist removed
        publish_deny_test(&topic, &user, &password, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    async fn blacklist_client_id_match_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();
        let client_id_prefix = format!("test_client_{}", unique_id());
        let client_id = format!("{}_{}", client_id_prefix, "001");

        // create user first
        create_user(&admin_client, &user, &password).await;

        // create blacklist with pattern match (e.g., "test_client_*")
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: format!("{}*", client_id_prefix),
            end_time: now_second() + 10000,
            desc: "ClientId pattern match test".to_string(),
        };

        create_blacklist_without_user(&admin_client, blacklist.clone()).await;

        let topic = unique_id();

        // publish deny true - should fail with matching client_id pattern
        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, true).await;

        // delete blacklist
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // publish deny false - should succeed after blacklist removed
        publish_deny_test_with_client_id(&topic, &user, &password, &client_id, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    async fn blacklist_user_match_test() {
        let admin_client = create_test_env().await;

        let user_prefix = format!("test_user_{}", unique_id());
        let user = format!("{}_{}", user_prefix, "001");
        let password: String = unique_id();

        // create user first
        create_user(&admin_client, &user, &password).await;

        // create blacklist with user pattern match (e.g., "test_user_*")
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::UserMatch,
            resource_name: format!("{}*", user_prefix),
            end_time: now_second() + 10000,
            desc: "User pattern match test".to_string(),
        };

        create_blacklist_without_user(&admin_client, blacklist.clone()).await;

        let topic = unique_id();

        // publish deny true - should fail with matching user pattern
        publish_deny_test(&topic, &user, &password, true).await;

        // delete blacklist
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // publish deny false - should succeed after blacklist removed
        publish_deny_test(&topic, &user, &password, false).await;

        delete_user(&admin_client, &user).await;
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn blacklist_ip_cidr_test() {
        let admin_client = create_test_env().await;

        let user = unique_id();
        let password: String = unique_id();

        // create user first
        create_user(&admin_client, &user, &password).await;

        // create blacklist for IP CIDR (127.0.0.0/24 covers 127.0.0.1)
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::IPCIDR,
            resource_name: "127.0.0.0/24".to_string(),
            end_time: now_second() + 10000,
            desc: "IP CIDR blacklist test".to_string(),
        };

        create_blacklist_without_user(&admin_client, blacklist.clone()).await;

        let topic = unique_id();

        // publish deny true - should fail from IP in blacklisted CIDR
        publish_deny_test(&topic, &user, &password, true).await;

        // delete blacklist
        delete_blacklist(&admin_client, blacklist.clone()).await;

        // publish deny false - should succeed after blacklist removed
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

        // publish retain
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

    async fn create_blacklist(admin_client: &AdminHttpClient, blacklist: MqttAclBlackList) {
        create_user(admin_client, &blacklist.resource_name, &blacklist.desc).await;

        let create_request = CreateBlackListReq {
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
            end_time: blacklist.end_time,
            desc: blacklist.desc,
        };

        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();
    }

    async fn delete_blacklist(admin_client: &AdminHttpClient, blacklist: MqttAclBlackList) {
        let delete_request = DeleteBlackListReq {
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
        };

        let res = admin_client.delete_blacklist(&delete_request).await;
        assert!(res.is_ok());
    }

    async fn create_user(admin_client: &AdminHttpClient, username: &str, password: &str) {
        let user = CreateUserReq {
            username: username.to_owned(),
            password: password.to_owned(),
            is_superuser: false,
        };

        let res = admin_client.create_user(&user).await;
        assert!(res.is_ok());
    }

    async fn delete_user(admin_client: &AdminHttpClient, username: &str) {
        let user = DeleteUserReq {
            username: username.to_owned(),
        };
        let res = admin_client.delete_user(&user).await;
        assert!(res.is_ok());
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

        // publish retain
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

    async fn create_blacklist_without_user(
        admin_client: &AdminHttpClient,
        blacklist: MqttAclBlackList,
    ) {
        let create_request = CreateBlackListReq {
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
            end_time: blacklist.end_time,
            desc: blacklist.desc,
        };

        admin_client
            .create_blacklist(&create_request)
            .await
            .unwrap();
    }
}
