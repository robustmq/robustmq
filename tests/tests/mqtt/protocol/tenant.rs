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
    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, build_v5_conn_pros, build_v5_pros,
            connect_server, create_test_env, distinct_conn, new_client, publish_data,
            subscribe_data_by_qos,
        },
        ClientTestProperties,
    };
    use admin_server::client::AdminHttpClient;
    use admin_server::mqtt::client::{ClientListReq, ClientListRow};
    use admin_server::mqtt::session::{SessionListReq, SessionListRow};
    use admin_server::mqtt::subscribe::{SubscribeListReq, SubscribeListRow};
    use admin_server::mqtt::tenant::{
        CreateMqttTenantReq, DeleteMqttTenantReq, MqttTenantListReq, MqttTenantListRow,
    };
    use admin_server::mqtt::topic::TopicListReq;
    use admin_server::tool::PageReplyData;
    use common_base::uuid::unique_id;
    use metadata_struct::mqtt::topic::Topic;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use paho_mqtt::{Message, MessageBuilder, PropertyCode};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn tenant_crud_test() {
        let admin_client = create_test_env().await;
        let tenant_name = format!("test_tenant_{}", unique_id());

        create_tenant(&admin_client, &tenant_name).await;
        sleep(Duration::from_millis(500)).await;
        assert_eq!(get_tenant(&admin_client, &tenant_name).await, 1);

        delete_tenant(&admin_client, &tenant_name).await;
        sleep(Duration::from_millis(500)).await;
        assert_eq!(get_tenant(&admin_client, &tenant_name).await, 0);
    }

    #[tokio::test]
    async fn default_tenant_pubsub_topic_test() {
        let admin_client = create_test_env().await;
        let topic = format!("/tenant_test/{}", unique_id());
        let client_id = build_client_id("default_tenant_pubsub");
        let message = "default_tenant_pubsub_test message".to_string();

        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            ..Default::default()
        });

        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.clone())
            .qos(1)
            .finalize();
        publish_data(&cli, msg, false);

        let call_fn = |msg: Message| String::from_utf8(msg.payload().to_vec()).unwrap() == message;
        assert!(subscribe_data_by_qos(&cli, &topic, 1, call_fn).is_ok());

        assert_online_resources_in_tenant(&admin_client, DEFAULT_TENANT, &client_id, &topic, false)
            .await;
        distinct_conn(cli);
        assert_persistent_resources_in_tenant(
            &admin_client,
            DEFAULT_TENANT,
            &client_id,
            &topic,
            false,
        )
        .await;
    }

    #[tokio::test]
    async fn custom_tenant_pubsub_via_user_property_test() {
        let admin_client = create_test_env().await;
        let tenant_name = format!("test_tenant_{}", unique_id());
        let topic = format!("/tenant_test/{}", unique_id());
        let client_id = build_client_id("custom_tenant_pubsub");
        let message = "custom_tenant_pubsub_test message".to_string();

        create_tenant(&admin_client, &tenant_name).await;
        sleep(Duration::from_millis(500)).await;

        let cli = {
            let mut props = build_v5_pros();
            props
                .push_string_pair(PropertyCode::UserProperty, "tenant", &tenant_name)
                .unwrap();
            let conn_opts = build_v5_conn_pros(
                ClientTestProperties {
                    mqtt_version: 5,
                    client_id: client_id.clone(),
                    addr: broker_addr_by_type("tcp"),
                    ..Default::default()
                },
                props,
                false,
            );
            let cli = new_client(&broker_addr_by_type("tcp"), &client_id);
            cli.connect(conn_opts).unwrap();
            cli
        };

        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.clone())
            .qos(1)
            .finalize();
        publish_data(&cli, msg, false);

        let call_fn = |msg: Message| String::from_utf8(msg.payload().to_vec()).unwrap() == message;
        assert!(subscribe_data_by_qos(&cli, &topic, 1, call_fn).is_ok());

        assert_online_resources_in_tenant(&admin_client, &tenant_name, &client_id, &topic, true)
            .await;
        distinct_conn(cli);
        assert_persistent_resources_in_tenant(
            &admin_client,
            &tenant_name,
            &client_id,
            &topic,
            true,
        )
        .await;

        delete_tenant(&admin_client, &tenant_name).await;
    }

    #[tokio::test]
    async fn custom_tenant_pubsub_via_username_prefix_test() {
        let admin_client = create_test_env().await;
        let tenant_name = format!("test_tenant_{}", unique_id());
        let topic = format!("/tenant_test/{}", unique_id());
        let client_id = build_client_id("custom_tenant_username");
        let message = "custom_tenant_username_test message".to_string();

        create_tenant(&admin_client, &tenant_name).await;
        sleep(Duration::from_millis(500)).await;

        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            user_name: format!("{}@admin", tenant_name),
            ..Default::default()
        });

        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.clone())
            .qos(1)
            .finalize();
        publish_data(&cli, msg, false);

        let call_fn = |msg: Message| String::from_utf8(msg.payload().to_vec()).unwrap() == message;
        assert!(subscribe_data_by_qos(&cli, &topic, 1, call_fn).is_ok());

        assert_online_resources_in_tenant(&admin_client, &tenant_name, &client_id, &topic, true)
            .await;
        distinct_conn(cli);
        assert_persistent_resources_in_tenant(
            &admin_client,
            &tenant_name,
            &client_id,
            &topic,
            true,
        )
        .await;

        delete_tenant(&admin_client, &tenant_name).await;
    }

    /// Assert that online resources (client, subscribe) exist under `tenant`.
    /// If `check_isolation` is true, also assert they are absent from DEFAULT_TENANT.
    async fn assert_online_resources_in_tenant(
        admin_client: &AdminHttpClient,
        tenant: &str,
        client_id: &str,
        topic: &str,
        check_isolation: bool,
    ) {
        // client_list.total_count is the global connection count, use data.len() for filtered results
        let client_res: PageReplyData<Vec<ClientListRow>> = admin_client
            .get_client_list(&ClientListReq {
                tenant: Some(tenant.to_string()),
                client_id: Some(client_id.to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(
            !client_res.data.is_empty(),
            "client {client_id} not found under tenant {tenant}"
        );

        let sub_count: PageReplyData<Vec<SubscribeListRow>> = admin_client
            .get_subscribe_list(&SubscribeListReq {
                tenant: Some(tenant.to_string()),
                client_id: Some(client_id.to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(
            sub_count.total_count >= 1,
            "subscribe for client {client_id} not found under tenant {tenant} topic {topic}"
        );

        if check_isolation {
            let client_default: PageReplyData<Vec<ClientListRow>> = admin_client
                .get_client_list(&ClientListReq {
                    tenant: Some(DEFAULT_TENANT.to_string()),
                    client_id: Some(client_id.to_string()),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert!(
                client_default.data.is_empty(),
                "client {client_id} should not appear under default tenant"
            );

            let sub_default: PageReplyData<Vec<SubscribeListRow>> = admin_client
                .get_subscribe_list(&SubscribeListReq {
                    tenant: Some(DEFAULT_TENANT.to_string()),
                    client_id: Some(client_id.to_string()),
                    ..Default::default()
                })
                .await
                .unwrap();
            println!(
                "[DEBUG] sub_default for client={client_id}: total_count={}, data={:?}",
                sub_default.total_count,
                sub_default
                    .data
                    .iter()
                    .map(|r| format!(
                        "tenant={} client_id={} path={}",
                        r.tenant, r.client_id, r.path
                    ))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                sub_default.total_count, 0,
                "subscribe for {client_id} should not appear under default tenant"
            );
        }
    }

    /// Assert that persistent resources (session, topic) exist under `tenant` after disconnect.
    /// If `check_isolation` is true, also assert they are absent from DEFAULT_TENANT.
    async fn assert_persistent_resources_in_tenant(
        admin_client: &AdminHttpClient,
        tenant: &str,
        client_id: &str,
        topic: &str,
        check_isolation: bool,
    ) {
        let session_count: PageReplyData<Vec<SessionListRow>> = admin_client
            .get_session_list(&SessionListReq {
                tenant: Some(tenant.to_string()),
                client_id: Some(client_id.to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(
            session_count.total_count >= 1,
            "session for {client_id} not found under tenant {tenant}"
        );

        let topic_count: PageReplyData<Vec<Topic>> = admin_client
            .get_topic_list(&TopicListReq {
                tenant: Some(tenant.to_string()),
                topic_name: Some(topic.to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(
            topic_count.total_count >= 1,
            "topic {topic} not found under tenant {tenant}"
        );

        if check_isolation {
            let session_default: PageReplyData<Vec<SessionListRow>> = admin_client
                .get_session_list(&SessionListReq {
                    tenant: Some(DEFAULT_TENANT.to_string()),
                    client_id: Some(client_id.to_string()),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(
                session_default.total_count, 0,
                "session for {client_id} should not appear under default tenant"
            );

            let topic_default: PageReplyData<Vec<Topic>> = admin_client
                .get_topic_list(&TopicListReq {
                    tenant: Some(DEFAULT_TENANT.to_string()),
                    topic_name: Some(topic.to_string()),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(
                topic_default.total_count, 0,
                "topic {topic} should not appear under default tenant"
            );
        }
    }

    async fn create_tenant(admin_client: &AdminHttpClient, tenant_name: &str) {
        admin_client
            .create_mqtt_tenant(&CreateMqttTenantReq {
                tenant_name: tenant_name.to_string(),
                desc: Some("integration test tenant".to_string()),
            })
            .await
            .unwrap();
    }

    async fn get_tenant(admin_client: &AdminHttpClient, tenant_name: &str) -> usize {
        let res: PageReplyData<Vec<MqttTenantListRow>> = admin_client
            .get_mqtt_tenant_list(&MqttTenantListReq {
                tenant_name: Some(tenant_name.to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        res.total_count
    }

    async fn delete_tenant(admin_client: &AdminHttpClient, tenant_name: &str) {
        admin_client
            .delete_mqtt_tenant(&DeleteMqttTenantReq {
                tenant_name: tenant_name.to_string(),
            })
            .await
            .unwrap();
    }
}
