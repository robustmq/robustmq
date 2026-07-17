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

use amqp_broker::core::cache::AmqpCacheManager;
use amqp_broker::storage::binding::BindingStorage;
use amqp_broker::storage::exchange::ExchangeStorage;
use amqp_broker::storage::queue::QueueStorage;
use broker_core::cache::NodeCacheManager;
use broker_core::cluster::ClusterStorage;
use broker_core::dynamic_config::build_cluster_config;
use broker_core::share_group::ShareGroupStorage;
use broker_core::tenant::TenantStorage;
use broker_core::topic::TopicStorage;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_security::manager::SecurityManager;
use common_security::storage::acl::AclStorage;
use common_security::storage::blacklist::BlackListStorage;
use common_security::storage::user::UserStorage;
use connector::manager::ConnectorManager;
use grpc_clients::meta::kafka::call::{
    list_kafka_delegation_token, list_kafka_quota, list_scram_credential,
};
use grpc_clients::pool::ClientPool;
use kafka_broker::core::cache::KafkaCacheManager;
use metadata_struct::kafka::delegation_token::KafkaDelegationToken;
use metadata_struct::kafka::quota::KafkaClientQuota;
use metadata_struct::kafka::scram::KafkaScramCredential;
use metadata_struct::tenant::DEFAULT_TENANT;
use mqtt_broker::core::cache::MQTTCacheManager;
use mqtt_broker::core::error::MqttBrokerError;
use mqtt_broker::core::tool::ResultMqttBrokerError;
use mqtt_broker::storage::auto_subscribe::AutoSubscribeStorage;
use mqtt_broker::storage::connector::ConnectorStorage;
use mqtt_broker::storage::schema::SchemaStorage;
use mqtt_broker::storage::topic_rewrite::TopicRewriteStorage;
use nats_broker::core::cache::NatsCacheManager;
use nats_broker::push::NatsSubscribeManager;
use nats_broker::storage::agent::Mq9AgentStorage;
use nats_broker::storage::mail::Mq9MailStorage;
use nats_broker::storage::subscribe::NatsSubscribeStorage;
use protocol::meta::meta_service_kafka::{
    ListKafkaDelegationTokenRequest, ListKafkaQuotaRequest, ListScramCredentialRequest,
};
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_engine::core::cache::StorageCacheManager;
use storage_engine::core::error::StorageEngineError;
use storage_engine::core::segment::{list_segment_metas, list_segments};
use storage_engine::core::shard::list_shards;
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub async fn load_metadata_cache(
    mqtt_cache_manager: &Arc<MQTTCacheManager>,
    nats_subscribe_manager: &Arc<NatsSubscribeManager>,
    nats_cache_manager: &Arc<NatsCacheManager>,
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
    security_manager: &Arc<SecurityManager>,
    kafka_cache: &Arc<KafkaCacheManager>,
    amqp_cache: &Arc<AmqpCacheManager>,
) -> ResultMqttBrokerError {
    info!("Starting to load metadata cache...");
    load_common_cache(
        &mqtt_cache_manager.node_cache,
        client_pool,
        connector_manager,
        schema_manager,
    )
    .await?;

    load_mqtt_cache(mqtt_cache_manager, security_manager, client_pool).await?;
    load_nats_cache(nats_subscribe_manager, nats_cache_manager, client_pool).await?;
    load_kafka_cache(kafka_cache, client_pool).await?;
    load_amqp_cache(amqp_cache, client_pool).await?;
    Ok(())
}

async fn load_amqp_cache(
    amqp_cache: &Arc<AmqpCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let storage = ExchangeStorage::new(client_pool.clone());
    let exchanges = storage
        .list_exchange_by_tenant(DEFAULT_TENANT)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
    for exchange in exchanges {
        amqp_cache.set_exchange(exchange);
    }

    let queue_storage = QueueStorage::new(client_pool.clone());
    let queues = queue_storage
        .list_queue_by_tenant(DEFAULT_TENANT)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
    for queue in queues {
        amqp_cache.set_queue(queue);
    }

    let binding_storage = BindingStorage::new(client_pool.clone());
    let bindings = binding_storage
        .list_binding_by_tenant(DEFAULT_TENANT)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
    for binding in bindings {
        amqp_cache.set_binding(binding);
    }
    Ok(())
}

async fn load_kafka_cache(
    kafka_cache: &Arc<KafkaCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let conf = broker_config();
    let reply = list_kafka_quota(
        client_pool,
        &conf.get_meta_service_addr(),
        ListKafkaQuotaRequest {
            tenant: DEFAULT_TENANT.to_string(),
        },
    )
    .await
    .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
    for raw in reply.quotas {
        let quota = KafkaClientQuota::decode(&raw)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        kafka_cache.set_quota(quota);
    }

    let reply = list_kafka_delegation_token(
        client_pool,
        &conf.get_meta_service_addr(),
        ListKafkaDelegationTokenRequest {
            tenant: DEFAULT_TENANT.to_string(),
        },
    )
    .await
    .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
    for raw in reply.tokens {
        let token = KafkaDelegationToken::decode(&raw)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        kafka_cache.set_delegation_token(token);
    }

    let reply = list_scram_credential(
        client_pool,
        &conf.get_meta_service_addr(),
        ListScramCredentialRequest {
            tenant: DEFAULT_TENANT.to_string(),
        },
    )
    .await
    .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
    for raw in reply.credentials {
        let credential = KafkaScramCredential::decode(&raw)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;
        kafka_cache.set_scram_credential(credential);
    }
    Ok(())
}

async fn load_common_cache(
    broker_cache: &Arc<NodeCacheManager>,
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
    schema_manager: &Arc<SchemaRegisterManager>,
) -> ResultMqttBrokerError {
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let nodes = cluster_storage
        .node_list()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load node list: {}", e)))?;
    for node in nodes.iter() {
        broker_cache.add_node(node.clone());
    }

    let cluster = build_cluster_config(client_pool).await.map_err(|e| {
        MqttBrokerError::CommonError(format!("Failed to load cluster config: {}", e))
    })?;
    broker_cache.set_cluster_config(cluster);

    let topic_storage = TopicStorage::new(client_pool.clone());
    let topic_list = topic_storage
        .all()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load topics: {}", e)))?;
    for topic in topic_list.iter() {
        broker_cache.add_topic(&topic.clone());
    }

    let connector_storage = ConnectorStorage::new(client_pool.clone());
    let connectors = connector_storage
        .list_all_connectors()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load connectors: {}", e)))?;
    for connector in connectors.iter() {
        connector_manager.add_connector(connector);
    }

    let schema_storage = SchemaStorage::new(client_pool.clone());
    let schemas = schema_storage
        .list(None, None)
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load schemas: {}", e)))?;
    for schema in schemas.iter() {
        schema_manager.add_schema(schema.clone());
    }

    let schema_storage = SchemaStorage::new(client_pool.clone());
    let schema_binds = schema_storage
        .list_bind(None)
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load schema binds: {}", e)))?;
    for schema in schema_binds.iter() {
        schema_manager.add_bind(schema);
    }

    let tenant_storage = TenantStorage::new(client_pool.clone());
    let tenants = tenant_storage
        .list_all()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load tenants: {}", e)))?;
    for tenant in tenants.iter() {
        broker_cache.add_tenant(tenant.clone());
    }

    let share_group_storage = ShareGroupStorage::new(client_pool.clone());
    let share_groups = share_group_storage
        .list_all()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load share groups: {}", e)))?;
    let share_group_count = share_groups.len();
    for group in share_groups {
        broker_cache.add_share_group(group);
    }

    let share_group_members = share_group_storage.list_all_members().await.map_err(|e| {
        MqttBrokerError::CommonError(format!("Failed to load share group members: {}", e))
    })?;
    let share_group_member_count = share_group_members.len();
    for member in share_group_members {
        broker_cache.add_share_group_member(&member);
    }

    info!(
        "Common cache loaded: nodes={}, topics={}, connectors={}, schemas={}, schema_binds={}, tenants={}, share_groups={}, share_group_members={}",
        nodes.len(),
        topic_list.len(),
        connectors.len(),
        schemas.len(),
        schema_binds.len(),
        tenants.len(),
        share_group_count,
        share_group_member_count,
    );

    Ok(())
}

async fn load_mqtt_cache(
    cache_manager: &Arc<MQTTCacheManager>,
    security_manager: &Arc<SecurityManager>,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let user_storage = UserStorage::new(client_pool.clone());
    let user_list = user_storage
        .user_list()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load users: {}", e)))?;
    for user in user_list.iter() {
        security_manager.metadata.add_user(user.clone());
    }

    let acl_storage = AclStorage::new(client_pool.clone());
    let acl_list = acl_storage
        .list_acl()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load ACLs: {}", e)))?;
    for acl in acl_list.iter() {
        security_manager.metadata.add_acl(acl.clone());
    }

    let blacklist_storage = BlackListStorage::new(client_pool.clone());
    let blacklist_list = blacklist_storage
        .list_blacklist()
        .await
        .map_err(|e| MqttBrokerError::CommonError(format!("Failed to load blacklist: {}", e)))?;
    for blacklist in blacklist_list.iter() {
        security_manager.metadata.add_blacklist(blacklist.clone());
    }

    let topic_storage = TopicRewriteStorage::new(client_pool.clone());
    let topic_rewrite_rules = topic_storage.all_topic_rewrite_rule().await.map_err(|e| {
        MqttBrokerError::CommonError(format!("Failed to load topic rewrite rules: {}", e))
    })?;
    for rule in topic_rewrite_rules.iter() {
        cache_manager.add_topic_rewrite_rule(rule.clone());
    }

    let auto_subscribe_storage = AutoSubscribeStorage::new(client_pool.clone());
    let auto_subscribe_rules = auto_subscribe_storage
        .list_auto_subscribe_rule(None)
        .await
        .map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to load auto subscribe rules: {}", e))
        })?;
    for rule in auto_subscribe_rules.iter() {
        cache_manager.add_auto_subscribe_rule(rule.clone());
    }

    info!(
        "MQTT cache loaded: users={}, acls={}, blacklist={}, topic_rewrite_rules={}, auto_subscribe_rules={}",
        user_list.len(),
        acl_list.len(),
        blacklist_list.len(),
        topic_rewrite_rules.len(),
        auto_subscribe_rules.len(),
    );

    Ok(())
}

pub async fn load_engine_cache(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), StorageEngineError> {
    for shard in list_shards(client_pool).await? {
        cache_manager.set_shard(shard);
    }

    for segment in list_segments(client_pool).await? {
        cache_manager.set_segment(&segment);
    }

    for meta in list_segment_metas(client_pool).await? {
        cache_manager.set_segment_meta(meta);
    }

    for shard in cache_manager.shards.iter() {
        cache_manager.sort_offset_index(&shard.shard_name);
    }

    info!(
        "Engine cache loaded: shards={}, segments={}, segment_metadatas={}",
        cache_manager.shards.len(),
        cache_manager.segments.len(),
        cache_manager.segment_metadatas.len(),
    );

    Ok(())
}

pub async fn load_nats_cache(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    cache_manager: &Arc<NatsCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), CommonError> {
    let subscribe_storage = NatsSubscribeStorage::new(client_pool.clone());
    let subscribes = subscribe_storage.list(0).await?;
    let subscribe_count = subscribes.len();
    for subscribe in subscribes {
        subscribe_manager.add_subscribe(subscribe);
    }

    let mail_storage = Mq9MailStorage::new(client_pool.clone());
    let mails = mail_storage.list("").await?;
    let mail_count = mails.len();
    for mail in mails {
        cache_manager.add_mail(mail);
    }

    let agent_storage = Mq9AgentStorage::new(client_pool.clone());
    let agents = agent_storage.list("").await?;
    let agent_count = agents.len();
    for agent in agents {
        cache_manager.add_agent(agent);
    }

    info!(
        "NATS cache loaded: subscribes={}, mails={}, agents={}",
        subscribe_count, mail_count, agent_count
    );
    Ok(())
}
