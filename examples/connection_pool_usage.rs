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

//! Example demonstrating the improved connection pool features
//! 
//! This example shows how to:
//! - Create a connection pool with custom timeout
//! - Warm up connections proactively
//! - Monitor connection pool health
//! - Handle unhealthy connections

use std::time::Duration;
use grpc_clients::pool::ClientPool;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Example 1: Create connection pool with custom timeout
    let client_pool = ClientPool::new_with_timeout(
        100,                           // max_open_connection
        Duration::from_secs(10),       // connection_timeout
    );
    info!("Created connection pool with 10s timeout");

    // Example 2: Warm up connection pools for known MQTT Broker addresses
    let broker_addrs = vec![
        "192.168.0.28:1228".to_string(),
        "192.168.0.29:1228".to_string(),
    ];

    info!("Warming up connection pools...");
    let warmup_results = client_pool.warmup_mqtt_broker_pools(&broker_addrs).await;
    
    for (addr, result) in warmup_results {
        match result {
            Ok(_) => info!("✓ Successfully warmed up pool for {}", addr),
            Err(e) => warn!("✗ Failed to warm up pool for {}: {}", addr, e),
        }
    }

    // Example 3: Monitor connection pool health
    info!("Checking connection pool health...");
    let health_statuses = client_pool.get_all_mqtt_broker_pool_health().await;
    
    for health in health_statuses {
        info!(
            "Pool {}: healthy={}, connections={}, in_use={}, idle={}",
            health.addr,
            health.is_healthy,
            health.connections,
            health.in_use,
            health.idle
        );
        
        // Example 4: Handle unhealthy connections
        if !health.is_healthy {
            warn!("Detected unhealthy pool at {}, clearing it", health.addr);
            client_pool.clear_mqtt_broker_pool(&health.addr);
            
            // Optionally, try to warm up again
            if let Err(e) = client_pool.warmup_mqtt_broker_pool(&health.addr).await {
                warn!("Failed to re-warm pool for {}: {}", health.addr, e);
            }
        }
    }

    // Example 5: Periodic health monitoring (background task)
    let pool_clone = client_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            let health_statuses = pool_clone.get_all_mqtt_broker_pool_health().await;
            let total_pools = health_statuses.len();
            let healthy_pools = health_statuses.iter().filter(|h| h.is_healthy).count();
            
            info!(
                "Connection pool health check: {}/{} pools healthy",
                healthy_pools, total_pools
            );
            
            for status in health_statuses {
                if !status.is_healthy {
                    warn!(
                        "⚠️  Unhealthy pool at {}: connections={}, idle={}",
                        status.addr, status.connections, status.idle
                    );
                }
            }
        }
    });

    // Example 6: Using the connection pool for actual requests
    // This is just a demonstration - in real code, you would make actual gRPC calls
    for addr in &broker_addrs {
        match client_pool.mqtt_broker_mqtt_services_client(addr).await {
            Ok(conn) => {
                info!("✓ Successfully got connection for {}", addr);
                // Use the connection for gRPC calls
                drop(conn); // Connection returns to pool when dropped
            }
            Err(e) => {
                warn!("✗ Failed to get connection for {}: {}", addr, e);
                
                // Check pool health for this specific address
                if let Some(health) = client_pool.get_mqtt_broker_pool_health(addr).await {
                    warn!("Pool health details: {:?}", health);
                }
            }
        }
    }

    // Keep the application running to observe periodic health checks
    info!("Example running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

