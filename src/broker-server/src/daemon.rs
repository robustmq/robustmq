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

use crate::connection::network_connection_gc;
use common_base::{node_status::NodeStatus, task::TaskKind};
use common_group::storage::start_offset_sync_task;
use common_security::sync::start_auth_sync_thread;
use connector::start_connector;
use delay_message::manager::start_delay_message_manager_thread;
use delay_task::start_delay_task_manager_thread;
use network_server::command::CommandRegistry;
use network_server::common::handler::handler_process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use system_info::{start_system_info_collection, start_tokio_runtime_info_collection};
use tokio::{sync::broadcast, time::sleep};
use tracing::{error, info};

use crate::BrokerServer;

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_term_signal(_sig: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

/// Register an OS-level SIGINT/SIGTERM handler via libc `sigaction`.
///
/// Uses libc rather than `tokio::signal` because with multiple Tokio runtimes
/// the per-runtime signal driver does not reliably receive process signals.
/// The handler only flips an atomic flag; `awaiting_stop` polls it.
pub(crate) fn register_shutdown_listener() {
    #[cfg(unix)]
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = handle_term_signal as extern "C" fn(libc::c_int) as usize;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = libc::SA_RESTART;
        if libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut()) != 0 {
            error!("failed to register SIGINT handler");
        }
        if libc::sigaction(libc::SIGTERM, &action, std::ptr::null_mut()) != 0 {
            error!("failed to register SIGTERM handler");
        }
    }

    // Watchdog: force-exit if graceful shutdown does not finish in time,
    // guaranteeing the process always terminates.
    std::thread::spawn(|| {
        while !SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(100));
        }
        info!("Shutdown requested, graceful shutdown has up to 20s before force-exit");
        std::thread::sleep(Duration::from_secs(20));
        error!("Graceful shutdown did not finish in 20s, forcing exit");
        std::process::exit(0);
    });
}

impl BrokerServer {
    pub fn start_broker_handler_pool(
        &self,
        commands: CommandRegistry,
        stop: broadcast::Sender<bool>,
    ) {
        handler_process(
            "broker-handler",
            self.config.broker_network.handler_thread_num,
            self.connection_manager.clone(),
            commands,
            self.shared_request_channel.clone(),
            self.task_supervisor.clone(),
            stop,
        );
    }

    pub fn start_node_call_manager(&self, stop: broadcast::Sender<bool>) {
        let node_call_manager = self.node_call_manager.clone();
        self.task_supervisor
            .spawn(TaskKind::BrokerNodeCall.to_string(), async move {
                node_call_manager.start(stop).await;
            });
    }

    pub async fn wait_for_node_call_manager_ready(&self) {
        loop {
            if self.node_call_manager.is_ready().await {
                return;
            }
            sleep(Duration::from_millis(5)).await;
        }
    }

    pub async fn start_background_services(
        &self,
        stop: broadcast::Sender<bool>,
        monitor_interval_ms: u64,
    ) {
        // delay task
        let rocksdb_engine_handler = self.rocksdb_engine_handler.clone();
        let broker_cache = self.broker_cache.clone();
        let delay_task_manager = self.delay_task_manager.clone();
        let node_call_manager = self.node_call_manager.clone();
        let task_supervisor = self.task_supervisor.clone();
        self.server_runtime.spawn(async move {
            if let Err(e) = start_delay_task_manager_thread(
                &rocksdb_engine_handler,
                &delay_task_manager,
                &broker_cache,
                &node_call_manager,
                &task_supervisor,
            )
            .await
            {
                error!("Failed to start DelayTask pop threads: {}", e);
                std::process::exit(1);
            }
        });

        // delay message
        let delay_message_manager = self.mqtt_params.delay_message_manager.clone();
        let task_supervisor = self.task_supervisor.clone();
        if let Err(e) =
            start_delay_message_manager_thread(&delay_message_manager, &task_supervisor).await
        {
            error!("Failed to start delay message manager, error:{}", e);
            std::process::exit(1);
        }

        // connection gc
        let connection_manager = self.connection_manager.clone();
        let tx = stop.clone();
        self.task_supervisor
            .spawn(TaskKind::NetworkConnectionGC.to_string(), async move {
                network_connection_gc(connection_manager, tx).await
            });

        // offset async commit
        let offset_manager = self.offset_manager.clone();
        let stop_send = stop.clone();
        self.task_supervisor
            .spawn(TaskKind::OffsetAsyncCommit.to_string(), async move {
                start_offset_sync_task(offset_manager, stop_send).await;
            });

        // sync auth info
        start_auth_sync_thread(
            self.mqtt_params.security_manager.clone(),
            self.task_supervisor.clone(),
            stop.clone(),
        );

        // system info collection
        let tx = stop.clone();
        self.task_supervisor
            .spawn(TaskKind::SystemInfoCollection.to_string(), async move {
                start_system_info_collection(tx, monitor_interval_ms).await;
            });

        // tokio runtime info collection
        let runtime_handles = vec![
            ("server".to_string(), self.server_runtime.handle().clone()),
            ("meta".to_string(), self.meta_runtime.handle().clone()),
            ("broker".to_string(), self.broker_runtime.handle().clone()),
        ];
        let tx = stop.clone();
        self.task_supervisor.spawn(
            TaskKind::TokioRuntimeInfoCollection.to_string(),
            async move {
                start_tokio_runtime_info_collection(runtime_handles, tx, monitor_interval_ms).await;
            },
        );

        // connector
        let message_storage = self.mqtt_params.storage_driver_manager.clone();
        let connector_manager = self.mqtt_params.connector_manager.clone();
        let client_pool = self.client_pool.clone();
        let task_supervisor = self.task_supervisor.clone();
        self.server_runtime.spawn(async move {
            start_connector(
                &client_pool,
                &message_storage,
                &connector_manager,
                &task_supervisor,
                &stop,
            )
            .await;
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn awaiting_stop(
        &self,
        broker_common_stop: broadcast::Sender<bool>,
        meta_stop: Option<broadcast::Sender<bool>>,
        network_handler_stop_send: broadcast::Sender<bool>,
        mqtt_stop: Option<broadcast::Sender<bool>>,
        kafka_stop: Option<broadcast::Sender<bool>>,
        amqp_stop: Option<broadcast::Sender<bool>>,
        nats_stop: Option<broadcast::Sender<bool>>,
        engine_stop: Option<broadcast::Sender<bool>>,
    ) {
        self.server_runtime.block_on(async {
            self.broker_cache.set_status(NodeStatus::Running).await;

            // Wait for the termination signal (set by the libc handler).
            while !SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                sleep(Duration::from_millis(100)).await;
            }

            info!("Termination signal received, the service starts to stop");

            self.broker_cache.set_status(NodeStatus::Stopping).await;

            // Stop Phase 1: Broker Network
            if let Err(e) = network_handler_stop_send.send(true) {
                error!("{}", e);
            }

            // Stop Phase 2: MQTT Broker
            if let Some(sx) = mqtt_stop {
                if let Err(e) = sx.send(true) {
                    error!("mqtt stop signal, error message:{}", e);
                }
            }

            // Stop Phase 3: NATS Broker
            if let Some(sx) = nats_stop {
                if let Err(e) = sx.send(true) {
                    error!("nats stop signal, error message:{}", e);
                }
            }

            // Stop Phase 4: Kafka Broker
            if let Some(sx) = kafka_stop {
                if let Err(e) = sx.send(true) {
                    error!("kafka stop signal, error message:{}", e);
                }
            }

            // Stop Phase 5: AMQP Broker
            if let Some(sx) = amqp_stop {
                if let Err(e) = sx.send(true) {
                    error!("amqp stop signal, error message:{}", e);
                }
            }

            // Stop Phase 6: Common
            if let Err(e) = self.delay_task_manager.stop().await {
                error!("delay task stop signal, error message{}", e);
            }

            // Stop Phase 7: Broker Common
            if let Err(e) = broker_common_stop.send(true) {
                error!("broker common stop signal, error message:{}", e);
            }

            // Stop Phase 8: Storage Engine
            if let Some(sx) = engine_stop {
                if let Err(e) = sx.send(true) {
                    error!("storage engine stop signal, error message:{}", e);
                }
            }

            let drain_start = tokio::time::Instant::now();
            while self.task_supervisor.has_running() {
                if drain_start.elapsed() >= Duration::from_secs(5) {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }

            // Stop Phase 9: Meta Service.
            // A restarting node keeps its Raft membership; per openraft, a node
            // that simply restarts needs no membership change.
            if let Some(sx) = meta_stop {
                if let Err(e) = sx.send(true) {
                    error!("meta stop signal, error message:{}", e);
                }
            }
        });
    }
}
