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

use std::sync::Arc;

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::exchange::{
    AMQPMethod, Bind, BindOk, Declare, DeclareOk, Delete, DeleteOk, Unbind, UnbindOk,
};
use amq_protocol::protocol::AMQPClass;
use metadata_struct::amqp::binding::{AmqpBinding, AmqpBindingDestinationType};
use metadata_struct::amqp::exchange::{AmqpExchange, AmqpExchangeType};
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::amqp::route;
use crate::core::cache::AmqpCacheManager;
use crate::storage::binding::BindingStorage;
use crate::storage::exchange::ExchangeStorage;

pub(crate) async fn process_exchange_full(
    channel_id: u16,
    method: &AMQPMethod,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Declare(declare) => {
            process_exchange_declare(
                channel_id,
                declare,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Delete(delete) => {
            process_exchange_delete(
                channel_id,
                delete,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Bind(bind) => {
            process_exchange_bind(
                channel_id,
                bind,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Unbind(unbind) => {
            process_exchange_unbind(
                channel_id,
                unbind,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        _ => None,
    }
}

async fn process_exchange_declare(
    channel_id: u16,
    declare: &Declare,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let exchange_name = declare.exchange.to_string();
    let exchange_type =
        AmqpExchangeType::from_str_opt(declare.kind.as_str()).unwrap_or_else(|| {
            warn!(
            "AMQP Exchange.Declare: unrecognized exchange type '{}' for {}, defaulting to direct",
            declare.kind.as_str(),
            exchange_name
        );
            AmqpExchangeType::Direct
        });
    let arguments = route::field_table_to_map(&declare.arguments);

    let tenant = amqp_cache.tenant_for(connection_id);
    let exchange = AmqpExchange::new(
        &tenant,
        &exchange_name,
        exchange_type,
        declare.durable,
        declare.auto_delete,
        declare.internal,
        arguments,
    );
    let storage = ExchangeStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    match storage.set_exchange(&exchange).await {
        Ok(()) => amqp_cache.set_exchange(exchange),
        Err(e) => warn!("AMQP Exchange.Declare failed for {}: {}", exchange_name, e),
    }

    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Exchange(AMQPMethod::DeclareOk(DeclareOk {})),
    ))
}

async fn process_exchange_delete(
    channel_id: u16,
    delete: &Delete,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let exchange_name = delete.exchange.to_string();
    let tenant = amqp_cache.tenant_for(connection_id);
    let storage = ExchangeStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    match storage.delete_exchange(&tenant, &exchange_name).await {
        Ok(()) => amqp_cache.remove_exchange(&tenant, &exchange_name),
        Err(e) => warn!("AMQP Exchange.Delete failed for {}: {}", exchange_name, e),
    }

    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Exchange(AMQPMethod::DeleteOk(DeleteOk {})),
    ))
}

async fn process_exchange_bind(
    channel_id: u16,
    bind: &Bind,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let tenant = amqp_cache.tenant_for(connection_id);
    let arguments = route::field_table_to_map(&bind.arguments);
    let binding = AmqpBinding::new(
        &tenant,
        bind.source.as_str(),
        bind.destination.as_str(),
        AmqpBindingDestinationType::Exchange,
        bind.routing_key.as_str(),
        arguments,
    );
    let storage = BindingStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    match storage.set_binding(&binding).await {
        Ok(()) => amqp_cache.set_binding(binding),
        Err(e) => warn!("AMQP Exchange.Bind failed: {}", e),
    }

    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Exchange(AMQPMethod::BindOk(BindOk {})),
    ))
}

async fn process_exchange_unbind(
    channel_id: u16,
    unbind: &Unbind,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let tenant = amqp_cache.tenant_for(connection_id);
    let storage = BindingStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    let destination_type = AmqpBindingDestinationType::Exchange;
    match storage
        .delete_binding(
            &tenant,
            unbind.source.as_str(),
            unbind.destination.as_str(),
            &destination_type,
            unbind.routing_key.as_str(),
        )
        .await
    {
        Ok(()) => {
            let key = format!(
                "{}/{}/{}/{}",
                unbind.source.as_str(),
                destination_type.as_str(),
                unbind.destination.as_str(),
                unbind.routing_key.as_str()
            );
            amqp_cache.remove_binding(&tenant, &key);
        }
        Err(e) => warn!("AMQP Exchange.Unbind failed: {}", e),
    }

    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Exchange(AMQPMethod::UnbindOk(UnbindOk {})),
    ))
}
