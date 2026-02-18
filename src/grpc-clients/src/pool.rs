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

use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::info;

const DEFAULT_CHANNELS_PER_ADDRESS: usize = 2000;

/// A pool of HTTP/2 channels to a single address.
/// Each channel is a separate TCP connection that supports HTTP/2 multiplexing
/// (up to ~200 concurrent streams per connection).
/// Round-robin distributes requests across channels.
struct ChannelPool {
    channels: Vec<Channel>,
    next: AtomicUsize,
}

impl ChannelPool {
    fn new(addr: &str, num_channels: usize) -> Self {
        let channels: Vec<Channel> = (0..num_channels)
            .map(|_| Self::create_channel(addr))
            .collect();
        info!(
            "Channel pool created for {} with {} channels (lazy connect)",
            addr, num_channels
        );
        ChannelPool {
            channels,
            next: AtomicUsize::new(0),
        }
    }

    fn create_channel(addr: &str) -> Channel {
        Channel::from_shared(format!("http://{}", addr))
            .expect("Invalid gRPC URI")
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .tcp_nodelay(true)
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .http2_adaptive_window(true)
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(60))
            .keep_alive_while_idle(true)
            .connect_lazy()
    }

    fn get(&self) -> Channel {
        let idx = self.next.fetch_add(1, Ordering::Relaxed) % self.channels.len();
        self.channels[idx].clone()
    }
}

/// gRPC client pool using HTTP/2 channel multiplexing.
///
/// Instead of creating one TCP connection per request (like a traditional connection pool),
/// this uses a small fixed number of HTTP/2 channels per address. Each channel supports
/// hundreds of concurrent requests via HTTP/2 multiplexing.
///
/// Usage:
/// ```ignore
/// let pool = ClientPool::new(4); // 4 channels per address
/// let channel = pool.get_channel("127.0.0.1:1228");
/// let client = MetaServiceServiceClient::new(channel);
/// ```
#[derive(Clone)]
pub struct ClientPool {
    channels_per_address: usize,
    channel_pools: Arc<DashMap<String, Arc<ChannelPool>>>,
    // leader cache for write requests (Raft leader routing)
    meta_service_leader_addr_caches: Arc<DashMap<String, String>>,
}

impl ClientPool {
    pub fn new(channels_per_address: usize) -> Self {
        let channels_per_address = if channels_per_address == 0 {
            DEFAULT_CHANNELS_PER_ADDRESS
        } else {
            channels_per_address
        };
        Self {
            channels_per_address,
            channel_pools: Arc::new(DashMap::with_capacity(8)),
            meta_service_leader_addr_caches: Arc::new(DashMap::with_capacity(2)),
        }
    }

    /// Get an HTTP/2 channel for the given address.
    /// Creates a new channel pool if one doesn't exist for this address.
    /// Channels are lazy-connected: the TCP connection is established on first use.
    pub fn get_channel(&self, addr: &str) -> Channel {
        if let Some(pool) = self.channel_pools.get(addr) {
            return pool.get();
        }
        let pool = Arc::new(ChannelPool::new(addr, self.channels_per_address));
        self.channel_pools.insert(addr.to_string(), pool.clone());
        pool.get()
    }

    // ----------leader cache management -------------
    pub fn get_leader_addr(&self, method: &str) -> Option<Ref<'_, String, String>> {
        self.meta_service_leader_addr_caches.get(method)
    }

    pub fn set_leader_addr(&self, method: String, leader_addr: String) {
        info!(
            "Update Leader cache for method={}, leader={}",
            method, leader_addr
        );
        self.meta_service_leader_addr_caches
            .insert(method, leader_addr);
    }

    pub fn clear_leader_cache(&self) {
        self.meta_service_leader_addr_caches.clear();
    }
}
