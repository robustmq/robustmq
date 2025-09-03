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
    use common_base::tools::{now_second, unique_id};
    use grpc_clients::mqtt::admin::call::{
        mqtt_broker_create_blacklist, mqtt_broker_create_user, mqtt_broker_delete_blacklist,
        mqtt_broker_delete_user, mqtt_broker_list_blacklist,
    };
    use grpc_clients::pool::ClientPool;
    use std::sync::Arc;

    use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
    use paho_mqtt::MessageBuilder;
    use protocol::broker::broker_mqtt_admin::{
        CreateBlacklistRequest, CreateUserRequest, DeleteBlacklistRequest, DeleteUserRequest,
        ListBlacklistRequest,
    };

    use crate::mqtt_protocol::common::{
        broker_addr_by_type, broker_grpc_addr, build_client_id, connect_server, distinct_conn,
        ssl_by_type, ws_by_type,
    };
    use crate::mqtt_protocol::ClientTestProperties;

    #[tokio::test]
    async fn blacklist_storage_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = format!("test_cluster_{}", unique_id());

        let user = unique_id();
        let password: String = unique_id();

        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: user.to_string(),
            end_time: 10000,
            desc: password.to_string(),
        };

        create_blacklist(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            blacklist.clone(),
        )
        .await;

        let list_request = ListBlacklistRequest {};
        let mut flag = false;
        match mqtt_broker_list_blacklist(&client_pool, &grpc_addr, list_request).await {
            Ok(data) => {
                println!("list blacklist: {:?}", data.blacklists.len());
                for raw in data.blacklists {
                    if raw.resource_name == blacklist.resource_name {
                        flag = true;
                        break;
                    }
                }
            }
            Err(e) => {
                panic!("list blacklist error: {e:?}");
            }
        };
        assert!(flag);

        delete_blacklist(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            blacklist.clone(),
        )
        .await;

        let mut flag = false;
        match mqtt_broker_list_blacklist(&client_pool, &grpc_addr, list_request).await {
            Ok(data) => {
                for raw in data.blacklists {
                    if raw.resource_name == blacklist.resource_name {
                        flag = true;
                        break;
                    }
                }
            }
            Err(e) => {
                panic!("list blacklist error: {e:?}");
            }
        };

        assert!(!flag);
    }

    #[tokio::test]
    async fn blacklist_user_auth_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let grpc_addr = vec![broker_grpc_addr()];

        let cluster_name: String = format!("test_cluster_{}", unique_id());
        let user = unique_id();
        let password: String = unique_id();

        // create blacklist
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: user.to_string(),
            end_time: now_second() + 10000,
            desc: password.to_string(),
        };

        create_blacklist(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            blacklist.clone(),
        )
        .await;

        let topic = unique_id();

        // push deny true
        publish_deny_test(&topic, &user, &password, true).await;

        // delete blacklist
        delete_blacklist(
            client_pool.clone(),
            grpc_addr.clone(),
            cluster_name.clone(),
            blacklist.clone(),
        )
        .await;

        // push deny false
        publish_deny_test(&topic, &user, &password, false).await;
        delete_user(&client_pool, &grpc_addr, &blacklist.resource_name).await;
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

    async fn create_blacklist(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        cluster_name: String,
        blacklist: MqttAclBlackList,
    ) {
        create_user(
            &client_pool,
            &grpc_addr,
            &blacklist.resource_name,
            &blacklist.desc,
        )
        .await;

        let create_request = CreateBlacklistRequest {
            cluster_name,
            blacklist: blacklist.encode().unwrap(),
        };

        let res = mqtt_broker_create_blacklist(&client_pool, &grpc_addr, create_request).await;
        assert!(res.is_ok());
    }

    async fn delete_blacklist(
        client_pool: Arc<ClientPool>,
        grpc_addr: Vec<String>,
        cluster_name: String,
        blacklist: MqttAclBlackList,
    ) {
        let delete_request = DeleteBlacklistRequest {
            cluster_name,
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
        };

        let res = mqtt_broker_delete_blacklist(&client_pool, &grpc_addr, delete_request).await;
        assert!(res.is_ok());
    }

    async fn create_user(
        client_pool: &Arc<ClientPool>,
        addrs: &[String],
        username: &str,
        password: &str,
    ) {
        let user = CreateUserRequest {
            username: username.to_owned(),
            password: password.to_owned(),
            is_superuser: false,
        };

        let res = mqtt_broker_create_user(client_pool, addrs, user.clone()).await;
        assert!(res.is_ok());
    }

    async fn delete_user(client_pool: &Arc<ClientPool>, addrs: &[String], username: &str) {
        let user = DeleteUserRequest {
            username: username.to_owned(),
        };
        let res = mqtt_broker_delete_user(client_pool, addrs, user.clone()).await;
        assert!(res.is_ok());
    }
}
