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
        publish_data,
    };
    use crate::mqtt::protocol::ClientTestProperties;
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::acl::{AclListReq, AclListRow, CreateAclReq, DeleteAclReq};
    use admin_server::mqtt::user::{CreateUserReq, DeleteUserReq};
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use common_base::uuid::unique_id;
    use metadata_struct::acl::mqtt_acl::MqttAcl;
    use paho_mqtt::MessageBuilder;

    #[tokio::test]
    async fn acl_storage_test() {
        let admin_client = create_test_env().await;
        let acl = create_test_acl(
            MqttAclResourceType::User,
            "acl_storage_test".to_string(),
            "tp-1".to_string(),
            MqttAclAction::Publish,
            MqttAclPermission::Deny,
        );
        create_acl(&admin_client, acl.clone()).await;

        check_acl_in_list(&admin_client, &acl, true).await;

        delete_acl(&admin_client, acl.clone()).await;

        check_acl_in_list(&admin_client, &acl, false).await;
    }

    #[tokio::test]
    async fn user_publish_authorization_test() {
        let topic = format!("{}/{}", "/user_publish_authorization_test", unique_id());

        run_authorization_test(MqttAclResourceType::User, topic).await;
    }

    #[tokio::test]
    async fn client_publish_authorization_test() {
        let topic = format!("{}/{}", "/client_publish_authorization_test", unique_id());

        run_authorization_test(MqttAclResourceType::ClientId, topic).await;
    }

    async fn run_authorization_test(resource_type: MqttAclResourceType, topic: String) {
        let admin_client = create_test_env().await;
        let username = unique_id();
        let password = "caclpublic".to_string();
        let client_id = build_client_id("client_publish_authorization_test");

        // Create test user
        create_user(&admin_client, username.clone(), password.clone()).await;

        // Test publishing without ACL
        match resource_type {
            MqttAclResourceType::User => {
                publish_user_acl_test(&topic, username.clone(), password.clone(), false).await;
            }
            MqttAclResourceType::ClientId => {
                publish_client_id_acl_test(&topic, &client_id, &username, &password, false).await;
            }
        }

        // Create ACL rule
        let acl = match resource_type {
            MqttAclResourceType::User => create_test_acl(
                resource_type,
                username.clone(),
                topic.clone(),
                MqttAclAction::Publish,
                MqttAclPermission::Deny,
            ),
            MqttAclResourceType::ClientId => create_test_acl(
                resource_type,
                client_id.clone(),
                topic.clone(),
                MqttAclAction::Publish,
                MqttAclPermission::Deny,
            ),
        };

        create_acl(&admin_client, acl.clone()).await;

        // Test publishing with ACL
        match resource_type {
            MqttAclResourceType::User => {
                publish_user_acl_test(&topic, username.clone(), password.clone(), true).await;
            }
            MqttAclResourceType::ClientId => {
                publish_client_id_acl_test(&topic, &client_id, &username, &password, true).await;
            }
        }

        // Delete ACL rule
        delete_acl(&admin_client, acl.clone()).await;

        // Test publishing after deleting ACL
        match resource_type {
            MqttAclResourceType::User => {
                publish_user_acl_test(&topic, username.clone(), password.clone(), false).await;
            }
            MqttAclResourceType::ClientId => {
                publish_client_id_acl_test(&topic, &client_id, &username, &password, false).await;
            }
        }

        // Clean up test user
        delete_user(&admin_client, username.clone()).await;
    }

    fn create_test_acl(
        resource_type: MqttAclResourceType,
        resource_name: String,
        topic: String,
        action: MqttAclAction,
        permission: MqttAclPermission,
    ) -> MqttAcl {
        MqttAcl {
            resource_type,
            resource_name,
            topic,
            ip: "*".to_string(),
            action,
            permission,
        }
    }

    async fn check_acl_in_list(
        admin_client: &AdminHttpClient,
        expected_acl: &MqttAcl,
        should_exist: bool,
    ) {
        let list_request = AclListReq {
            limit: Some(10000),
            page: Some(1),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_acl_list::<AclListReq, Vec<AclListRow>>(&list_request)
            .await
        {
            Ok(page_data) => {
                let mut found = false;
                for acl_row in page_data.data {
                    if acl_row.resource_type == expected_acl.resource_type.to_string()
                        && acl_row.resource_name == expected_acl.resource_name
                        && acl_row.topic == expected_acl.topic
                        && acl_row.ip == expected_acl.ip
                        && acl_row.action == expected_acl.action.to_string()
                        && acl_row.permission == expected_acl.permission.to_string()
                    {
                        found = true;
                        break;
                    }
                }
                assert_eq!(
                    found,
                    should_exist,
                    "ACL {} in list",
                    if should_exist {
                        "should be"
                    } else {
                        "should not be"
                    }
                );
            }
            Err(e) => {
                panic!("list acl error: {e:?}");
            }
        }
    }
    async fn publish_user_acl_test(topic: &str, username: String, password: String, is_err: bool) {
        let protocol = 5;
        let network = "tcp";
        let qos = 1;
        let client_id =
            build_client_id(format!("publish_user_acl_test_{protocol}_{network}_{qos}").as_str());

        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            user_name: username.clone(),
            password: password.clone(),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        // publish
        let message = "publish_user_acl_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.to_owned())
            .qos(qos)
            .finalize();
        publish_data(&cli, msg, is_err);
        distinct_conn(cli);
    }

    async fn publish_client_id_acl_test(
        topic: &str,
        client_id: &str,
        username: &str,
        password: &str,
        is_err: bool,
    ) {
        let protocol = 5;
        let network = "tcp";
        let qos = 1;

        let client_properties = ClientTestProperties {
            mqtt_version: protocol,
            client_id: client_id.to_owned(),
            addr: broker_addr_by_type(network),
            user_name: username.to_owned(),
            password: password.to_owned(),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);

        // publish
        let message = "publish_client_id_acl_test mqtt message".to_string();
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.to_owned())
            .qos(qos)
            .retained(true)
            .finalize();

        publish_data(&cli, msg, is_err);
        distinct_conn(cli);
    }

    async fn create_user(admin_client: &AdminHttpClient, username: String, password: String) {
        let user = CreateUserReq {
            username,
            password,
            is_superuser: false,
        };
        let res = admin_client.create_user(&user).await;
        assert!(res.is_ok());
    }

    async fn delete_user(admin_client: &AdminHttpClient, username: String) {
        let user = DeleteUserReq { username };
        let res = admin_client.delete_user(&user).await;
        assert!(res.is_ok());
    }

    async fn create_acl(admin_client: &AdminHttpClient, acl: MqttAcl) {
        let create_request = CreateAclReq {
            resource_type: acl.resource_type.to_string(),
            resource_name: acl.resource_name,
            topic: acl.topic,
            ip: acl.ip,
            action: acl.action.to_string(),
            permission: acl.permission.to_string(),
        };

        let res = admin_client.create_acl(&create_request).await;
        assert!(res.is_ok());
    }

    async fn delete_acl(admin_client: &AdminHttpClient, acl: MqttAcl) {
        let delete_request = DeleteAclReq {
            resource_type: acl.resource_type.to_string(),
            resource_name: acl.resource_name,
            topic: acl.topic,
            ip: acl.ip,
            action: acl.action.to_string(),
            permission: acl.permission.to_string(),
        };

        let res = admin_client.delete_acl(&delete_request).await;
        assert!(res.is_ok());
    }
}
