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

//! Network Server Metrics
//! 
//! This module provides comprehensive monitoring metrics for the network server,
//! including connection metrics, packet processing metrics, performance metrics,
//! thread pool metrics, memory metrics, error metrics, and health metrics.

use crate::{
    counter_metric_inc, gauge_metric_inc, gauge_metric_inc_by, gauge_metrics_set, histogram_metric_observe,
    register_counter_metric, register_gauge_metric, register_histogram_metric_ms_with_default_buckets,
};
use crate::core::server::NoLabelSet;
use prometheus_client::encoding::EncodeLabelSet;
use std::sync::LazyLock;

// ================================================================================================
// Label Definitions
// ================================================================================================

/// Network protocol label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct NetworkProtocolLabel {
    pub protocol: String, // tcp, tls, websocket, quic
}

/// Network protocol with version label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct NetworkProtocolVersionLabel {
    pub protocol: String, // mqtt, kafka
    pub version: String,  // 3, 4, 5 for MQTT; 0.10, 2.0 for Kafka
}

/// Connection close reason label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ConnectionCloseLabel {
    pub protocol: String,
    pub reason: String, // client_close, server_close, timeout, error
}

/// Packet error label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct PacketErrorLabel {
    pub protocol: String,
    pub error_type: String, // decode, encode, timeout, invalid
}

/// Queue type label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct QueueTypeLabel {
    pub queue_type: String,   // request_main, request_child, response_main, response_child
    pub network_type: String, // tcp, websocket, quic
}

/// Thread type label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ThreadTypeLabel {
    pub thread_type: String,  // acceptor, handler, response
    pub network_type: String, // tcp, websocket, quic
}

/// Error type label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ErrorTypeLabel {
    pub error_type: String, // timeout, refused, reset, broken_pipe
    pub protocol: String,
}

/// Packet type label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct PacketTypeLabel {
    pub protocol: String,    // mqtt, kafka
    pub packet_type: String, // connect, publish, subscribe, etc.
}

/// Direction label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DirectionLabel {
    pub direction: String, // inbound, outbound
    pub protocol: String,
}

/// Server status label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ServerStatusLabel {
    pub status: String, // starting, running, stopping, stopped
}

/// Load level label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct LoadLevelLabel {
    pub level: String, // low, medium, high, critical
}

/// Component label for backpressure
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ComponentLabel {
    pub component: String, // acceptor, handler, response
}

/// Percentile label
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct PercentileLabel {
    pub percentile: String, // 50, 95, 99
    pub protocol: String,
}

// ================================================================================================
// Connection Metrics
// ================================================================================================

/// Current active connections by protocol
static NETWORK_ACTIVE_CONNECTIONS: LazyLock<
    crate::core::gauge::FamilyGauge<NetworkProtocolLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_active_connections_total",
        "Current number of active network connections by protocol",
    )
});

/// Total connections established
static NETWORK_CONNECTIONS_TOTAL: LazyLock<
    crate::core::counter::FamilyCounter<NetworkProtocolLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_connections_total",
        "Total number of network connections established",
    )
});

/// Connection establishment rate
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_CONNECTION_RATE,
    "network_connection_rate",
    "Rate of new connections per second",
    NetworkProtocolLabel
);

/// Connections closed by reason
static NETWORK_CONNECTIONS_CLOSED: LazyLock<
    crate::core::counter::FamilyCounter<ConnectionCloseLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_connections_closed_total",
        "Total number of connections closed by reason",
    )
});

/// Connection duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_CONNECTION_DURATION,
    "network_connection_duration_seconds",
    "Duration of network connections in seconds",
    NetworkProtocolLabel
);

/// Connection establishment duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_CONNECTION_ESTABLISHMENT_DURATION,
    "network_connection_establishment_duration_ms",
    "Time taken to establish network connections in milliseconds",
    NetworkProtocolLabel
);

/// TLS handshake duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_TLS_HANDSHAKE_DURATION,
    "network_tls_handshake_duration_ms",
    "Time taken for TLS handshake in milliseconds",
    NetworkProtocolLabel
);

// ================================================================================================
// Packet Processing Metrics
// ================================================================================================

/// Packets received total
static NETWORK_PACKETS_RECEIVED: LazyLock<
    crate::core::counter::FamilyCounter<NetworkProtocolVersionLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_packets_received_total",
        "Total number of packets received by protocol and version",
    )
});

/// Packets sent total
static NETWORK_PACKETS_SENT: LazyLock<
    crate::core::counter::FamilyCounter<NetworkProtocolVersionLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_packets_sent_total",
        "Total number of packets sent by protocol and version",
    )
});

/// Packet processing errors
static NETWORK_PACKET_ERRORS: LazyLock<
    crate::core::counter::FamilyCounter<PacketErrorLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_packet_errors_total",
        "Total number of packet processing errors",
    )
});

/// Packets dropped
static NETWORK_PACKETS_DROPPED: LazyLock<
    crate::core::counter::FamilyCounter<PacketErrorLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_packets_dropped_total",
        "Total number of packets dropped",
    )
});

/// Received packet size distribution
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_PACKET_SIZE_RECEIVED,
    "network_packet_size_bytes_received",
    "Size distribution of received packets in bytes",
    NetworkProtocolVersionLabel
);

/// Sent packet size distribution
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_PACKET_SIZE_SENT,
    "network_packet_size_bytes_sent",
    "Size distribution of sent packets in bytes",
    NetworkProtocolVersionLabel
);

/// Network throughput
static NETWORK_THROUGHPUT: LazyLock<
    crate::core::gauge::FamilyGauge<DirectionLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_throughput_bytes_per_second",
        "Network throughput in bytes per second",
    )
});

// ================================================================================================
// Latency and Performance Metrics
// ================================================================================================

/// Request total processing duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_REQUEST_DURATION,
    "network_request_duration_ms",
    "Total request processing duration from receive to response",
    PacketTypeLabel
);

/// Request queue waiting duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_REQUEST_QUEUE_DURATION,
    "network_request_queue_duration_ms",
    "Time requests spend waiting in queues",
    QueueTypeLabel
);

/// Business logic processing duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_REQUEST_HANDLER_DURATION,
    "network_request_handler_duration_ms",
    "Time spent in business logic processing",
    PacketTypeLabel
);

/// Response sending duration
register_histogram_metric_ms_with_default_buckets!(
    NETWORK_RESPONSE_SEND_DURATION,
    "network_response_send_duration_ms",
    "Time taken to send responses",
    NetworkProtocolLabel
);

/// Current queue sizes
static NETWORK_QUEUE_SIZE: LazyLock<
    crate::core::gauge::FamilyGauge<QueueTypeLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_queue_size",
        "Current size of various processing queues",
    )
});

/// Queue full events
static NETWORK_QUEUE_FULL: LazyLock<
    crate::core::counter::FamilyCounter<QueueTypeLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_queue_full_total",
        "Total number of times queues became full",
    )
});

/// Queue processing rate
static NETWORK_QUEUE_PROCESSING_RATE: LazyLock<
    crate::core::gauge::FamilyGauge<QueueTypeLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_queue_processing_rate",
        "Rate of queue processing (items per second)",
    )
});

// ================================================================================================
// Thread Pool Metrics
// ================================================================================================

/// Thread count by type
static NETWORK_THREAD_COUNT: LazyLock<
    crate::core::gauge::FamilyGauge<ThreadTypeLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_thread_count",
        "Number of threads by type and network type",
    )
});

/// Active thread count
static NETWORK_ACTIVE_THREADS: LazyLock<
    crate::core::gauge::FamilyGauge<ThreadTypeLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_active_threads",
        "Number of currently active threads",
    )
});

/// Thread utilization ratio
static NETWORK_THREAD_UTILIZATION: LazyLock<
    crate::core::gauge::FamilyGauge<ThreadTypeLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_thread_utilization_ratio",
        "Thread utilization ratio (0.0 to 1.0)",
    )
});

/// Thread lifecycle events
static NETWORK_THREAD_LIFECYCLE: LazyLock<
    crate::core::counter::FamilyCounter<ThreadTypeLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_thread_lifecycle_total",
        "Total thread creation and destruction events",
    )
});

// ================================================================================================
// Memory and Resource Metrics
// ================================================================================================

/// Connection manager memory usage
register_gauge_metric!(
    NETWORK_CONNECTION_MANAGER_MEMORY,
    "network_connection_manager_memory_bytes",
    "Memory used by connection manager in bytes",
    NoLabelSet
);

/// Queue memory usage
static NETWORK_QUEUE_MEMORY: LazyLock<
    crate::core::gauge::FamilyGauge<QueueTypeLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_queue_memory_bytes",
        "Memory used by queues in bytes",
    )
});

/// Codec buffer memory usage
static NETWORK_CODEC_BUFFER_MEMORY: LazyLock<
    crate::core::gauge::FamilyGauge<NetworkProtocolVersionLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_codec_buffer_bytes",
        "Memory used by codec buffers in bytes",
    )
});

/// Write buffer sizes
static NETWORK_WRITE_BUFFER_SIZE: LazyLock<
    crate::core::gauge::FamilyGauge<NetworkProtocolLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_write_buffer_bytes",
        "Size of write buffers in bytes",
    )
});

/// File descriptors used
register_gauge_metric!(
    NETWORK_FILE_DESCRIPTORS_USED,
    "network_file_descriptors_used",
    "Number of file descriptors currently in use",
    NoLabelSet
);

/// File descriptor utilization ratio
register_gauge_metric!(
    NETWORK_FILE_DESCRIPTORS_UTILIZATION,
    "network_file_descriptors_utilization_ratio",
    "File descriptor utilization ratio (0.0 to 1.0)",
    NoLabelSet
);

// ================================================================================================
// Error and Exception Metrics
// ================================================================================================

/// Connection errors
static NETWORK_CONNECTION_ERRORS: LazyLock<
    crate::core::counter::FamilyCounter<ErrorTypeLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_connection_errors_total",
        "Total number of connection errors by type",
    )
});

/// Codec errors
static NETWORK_CODEC_ERRORS: LazyLock<
    crate::core::counter::FamilyCounter<PacketErrorLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_codec_errors_total",
        "Total number of codec errors",
    )
});

/// Write failures
static NETWORK_WRITE_FAILURES: LazyLock<
    crate::core::counter::FamilyCounter<ErrorTypeLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_write_failures_total",
        "Total number of write failures",
    )
});

/// Protocol violations
static NETWORK_PROTOCOL_VIOLATIONS: LazyLock<
    crate::core::counter::FamilyCounter<PacketErrorLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_protocol_violations_total",
        "Total number of protocol violations",
    )
});

/// System call errors
static NETWORK_SYSCALL_ERRORS: LazyLock<
    crate::core::counter::FamilyCounter<ErrorTypeLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_syscall_errors_total",
        "Total number of system call errors",
    )
});

/// Memory allocation failures
register_counter_metric!(
    NETWORK_MEMORY_ALLOCATION_FAILURES,
    "network_memory_allocation_failures_total",
    "Total number of memory allocation failures",
    NoLabelSet
);

/// Thread creation failures
static NETWORK_THREAD_CREATION_FAILURES: LazyLock<
    crate::core::counter::FamilyCounter<ThreadTypeLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "network_thread_creation_failures_total",
        "Total number of thread creation failures",
    )
});

// ================================================================================================
// Business Logic Metrics
// ================================================================================================

/// MQTT connect attempts
static MQTT_CONNECT_ATTEMPTS: LazyLock<
    crate::core::counter::FamilyCounter<PacketErrorLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "mqtt_connect_attempts_total",
        "Total MQTT connection attempts",
    )
});

/// Active MQTT subscriptions
register_gauge_metric!(
    MQTT_SUBSCRIPTIONS_ACTIVE,
    "mqtt_subscriptions_active",
    "Number of active MQTT subscriptions",
    NoLabelSet
);

/// Retained messages count
register_gauge_metric!(
    MQTT_RETAINED_MESSAGES,
    "mqtt_retained_messages_count",
    "Number of retained MQTT messages",
    NoLabelSet
);

/// QoS distribution
static MQTT_QOS_DISTRIBUTION: LazyLock<
    crate::core::counter::FamilyCounter<PacketErrorLabel>,
> = LazyLock::new(|| {
    crate::core::counter::register_int_counter_family(
        "mqtt_qos_distribution",
        "Distribution of MQTT messages by QoS level",
    )
});

/// Kafka produce requests
register_counter_metric!(
    KAFKA_PRODUCE_REQUESTS,
    "kafka_produce_requests_total",
    "Total Kafka produce requests",
    NoLabelSet
);

/// Kafka consume requests
register_counter_metric!(
    KAFKA_CONSUME_REQUESTS,
    "kafka_consume_requests_total",
    "Total Kafka consume requests",
    NoLabelSet
);

/// Kafka partition assignments
register_counter_metric!(
    KAFKA_PARTITION_ASSIGNMENTS,
    "kafka_partition_assignments_total",
    "Total Kafka partition assignments",
    NoLabelSet
);

// ================================================================================================
// Server Health Metrics
// ================================================================================================

/// Server start time
register_gauge_metric!(
    NETWORK_SERVER_START_TIME,
    "network_server_start_time_seconds",
    "Server start time in Unix timestamp",
    NoLabelSet
);

/// Server uptime
register_gauge_metric!(
    NETWORK_SERVER_UPTIME,
    "network_server_uptime_seconds",
    "Server uptime in seconds",
    NoLabelSet
);

/// Server status
static NETWORK_SERVER_STATUS: LazyLock<
    crate::core::gauge::FamilyGauge<ServerStatusLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_server_status",
        "Current server status (1 for active status, 0 for inactive)",
    )
});

/// Graceful shutdown progress
register_gauge_metric!(
    NETWORK_GRACEFUL_SHUTDOWN_PROGRESS,
    "network_graceful_shutdown_progress_ratio",
    "Progress of graceful shutdown (0.0 to 1.0)",
    NoLabelSet
);

/// CPU usage ratio
register_gauge_metric!(
    NETWORK_SERVER_CPU_USAGE,
    "network_server_cpu_usage_ratio",
    "CPU usage ratio for network server (0.0 to 1.0)",
    NoLabelSet
);

/// Current load level
static NETWORK_SERVER_LOAD_LEVEL: LazyLock<
    crate::core::gauge::FamilyGauge<LoadLevelLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_server_load_level",
        "Current server load level (1 for active level, 0 for inactive)",
    )
});

/// Backpressure active
static NETWORK_BACKPRESSURE_ACTIVE: LazyLock<
    crate::core::gauge::FamilyGauge<ComponentLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_backpressure_active",
        "Whether backpressure is active for components (1 for active, 0 for inactive)",
    )
});

// ================================================================================================
// SLA Metrics
// ================================================================================================

/// Service availability ratio
register_gauge_metric!(
    NETWORK_SERVICE_AVAILABILITY,
    "network_service_availability_ratio",
    "Service availability ratio (0.0 to 1.0)",
    NoLabelSet
);

/// Request success rate
static NETWORK_REQUEST_SUCCESS_RATE: LazyLock<
    crate::core::gauge::FamilyGauge<NetworkProtocolVersionLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_request_success_rate",
        "Request success rate by protocol (0.0 to 1.0)",
    )
});

/// Request latency percentiles
static NETWORK_REQUEST_LATENCY_PERCENTILE: LazyLock<
    crate::core::gauge::FamilyGauge<PercentileLabel>,
> = LazyLock::new(|| {
    crate::core::gauge::register_int_gauge_family(
        "network_request_latency_percentile",
        "Request latency percentiles in milliseconds",
    )
});

// ================================================================================================
// Helper Functions
// ================================================================================================

/// Record connection established
pub fn record_connection_established(protocol: &str) {
    let label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    gauge_metric_inc!(NETWORK_ACTIVE_CONNECTIONS, label);
    counter_metric_inc!(NETWORK_CONNECTIONS_TOTAL, label);
}

/// Record connection closed
pub fn record_connection_closed(protocol: &str, reason: &str) {
    let active_label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    let close_label = ConnectionCloseLabel {
        protocol: protocol.to_string(),
        reason: reason.to_string(),
    };
    gauge_metric_inc_by!(NETWORK_ACTIVE_CONNECTIONS, active_label, -1);
    counter_metric_inc!(NETWORK_CONNECTIONS_CLOSED, close_label);
}

/// Record connection duration
pub fn record_connection_duration(protocol: &str, duration_ms: f64) {
    let label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    histogram_metric_observe!(NETWORK_CONNECTION_DURATION, duration_ms, label);
}

/// Record connection establishment duration
pub fn record_connection_establishment_duration(protocol: &str, duration_ms: f64) {
    let label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    histogram_metric_observe!(NETWORK_CONNECTION_ESTABLISHMENT_DURATION, duration_ms, label);
}

/// Record TLS handshake duration
pub fn record_tls_handshake_duration(protocol: &str, duration_ms: f64) {
    let label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    histogram_metric_observe!(NETWORK_TLS_HANDSHAKE_DURATION, duration_ms, label);
}

/// Record packet received
pub fn record_packet_received(protocol: &str, version: &str, size_bytes: f64) {
    let label = NetworkProtocolVersionLabel {
        protocol: protocol.to_string(),
        version: version.to_string(),
    };
    counter_metric_inc!(NETWORK_PACKETS_RECEIVED, label);
    histogram_metric_observe!(NETWORK_PACKET_SIZE_RECEIVED, size_bytes, label);
}

/// Record packet sent
pub fn record_packet_sent(protocol: &str, version: &str, size_bytes: f64) {
    let label = NetworkProtocolVersionLabel {
        protocol: protocol.to_string(),
        version: version.to_string(),
    };
    counter_metric_inc!(NETWORK_PACKETS_SENT, label);
    histogram_metric_observe!(NETWORK_PACKET_SIZE_SENT, size_bytes, label);
}

/// Record packet error
pub fn record_packet_error(protocol: &str, error_type: &str) {
    let label = PacketErrorLabel {
        protocol: protocol.to_string(),
        error_type: error_type.to_string(),
    };
    counter_metric_inc!(NETWORK_PACKET_ERRORS, label);
}

/// Record packet dropped
pub fn record_packet_dropped(protocol: &str, reason: &str) {
    let label = PacketErrorLabel {
        protocol: protocol.to_string(),
        error_type: reason.to_string(),
    };
    counter_metric_inc!(NETWORK_PACKETS_DROPPED, label);
}

/// Record request duration
pub fn record_request_duration(protocol: &str, packet_type: &str, duration_ms: f64) {
    let label = PacketTypeLabel {
        protocol: protocol.to_string(),
        packet_type: packet_type.to_string(),
    };
    histogram_metric_observe!(NETWORK_REQUEST_DURATION, duration_ms, label);
}

/// Record request queue duration
pub fn record_request_queue_duration(queue_type: &str, network_type: &str, duration_ms: f64) {
    let label = QueueTypeLabel {
        queue_type: queue_type.to_string(),
        network_type: network_type.to_string(),
    };
    histogram_metric_observe!(NETWORK_REQUEST_QUEUE_DURATION, duration_ms, label);
}

/// Record request handler duration
pub fn record_request_handler_duration(protocol: &str, packet_type: &str, duration_ms: f64) {
    let label = PacketTypeLabel {
        protocol: protocol.to_string(),
        packet_type: packet_type.to_string(),
    };
    histogram_metric_observe!(NETWORK_REQUEST_HANDLER_DURATION, duration_ms, label);
}

/// Record response send duration
pub fn record_response_send_duration(protocol: &str, duration_ms: f64) {
    let label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    histogram_metric_observe!(NETWORK_RESPONSE_SEND_DURATION, duration_ms, label);
}

/// Set queue size
pub fn set_queue_size(queue_type: &str, network_type: &str, size: i64) {
    let label = QueueTypeLabel {
        queue_type: queue_type.to_string(),
        network_type: network_type.to_string(),
    };
    gauge_metrics_set!(NETWORK_QUEUE_SIZE, label, size);
}

/// Record queue full event
pub fn record_queue_full(queue_type: &str, network_type: &str) {
    let label = QueueTypeLabel {
        queue_type: queue_type.to_string(),
        network_type: network_type.to_string(),
    };
    counter_metric_inc!(NETWORK_QUEUE_FULL, label);
}

/// Set queue processing rate
pub fn set_queue_processing_rate(queue_type: &str, network_type: &str, rate: f64) {
    let label = QueueTypeLabel {
        queue_type: queue_type.to_string(),
        network_type: network_type.to_string(),
    };
    gauge_metrics_set!(NETWORK_QUEUE_PROCESSING_RATE, label, rate as i64);
}

/// Set thread count
pub fn set_thread_count(thread_type: &str, network_type: &str, count: i64) {
    let label = ThreadTypeLabel {
        thread_type: thread_type.to_string(),
        network_type: network_type.to_string(),
    };
    gauge_metrics_set!(NETWORK_THREAD_COUNT, label, count);
}

/// Set active thread count
pub fn set_active_thread_count(thread_type: &str, network_type: &str, count: i64) {
    let label = ThreadTypeLabel {
        thread_type: thread_type.to_string(),
        network_type: network_type.to_string(),
    };
    gauge_metrics_set!(NETWORK_ACTIVE_THREADS, label, count);
}

/// Set thread utilization
pub fn set_thread_utilization(thread_type: &str, network_type: &str, ratio: f64) {
    let label = ThreadTypeLabel {
        thread_type: thread_type.to_string(),
        network_type: network_type.to_string(),
    };
    gauge_metrics_set!(NETWORK_THREAD_UTILIZATION, label, (ratio * 1000.0) as i64);
}

/// Record thread lifecycle event
pub fn record_thread_lifecycle(thread_type: &str, network_type: &str) {
    let label = ThreadTypeLabel {
        thread_type: thread_type.to_string(),
        network_type: network_type.to_string(),
    };
    counter_metric_inc!(NETWORK_THREAD_LIFECYCLE, label);
}

/// Set connection manager memory usage
pub fn set_connection_manager_memory(bytes: i64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_CONNECTION_MANAGER_MEMORY, label, bytes);
}

/// Set queue memory usage
pub fn set_queue_memory(queue_type: &str, network_type: &str, bytes: i64) {
    let label = QueueTypeLabel {
        queue_type: queue_type.to_string(),
        network_type: network_type.to_string(),
    };
    gauge_metrics_set!(NETWORK_QUEUE_MEMORY, label, bytes);
}

/// Set codec buffer memory usage
pub fn set_codec_buffer_memory(protocol: &str, version: &str, bytes: i64) {
    let label = NetworkProtocolVersionLabel {
        protocol: protocol.to_string(),
        version: version.to_string(),
    };
    gauge_metrics_set!(NETWORK_CODEC_BUFFER_MEMORY, label, bytes);
}

/// Set write buffer size
pub fn set_write_buffer_size(protocol: &str, bytes: i64) {
    let label = NetworkProtocolLabel {
        protocol: protocol.to_string(),
    };
    gauge_metrics_set!(NETWORK_WRITE_BUFFER_SIZE, label, bytes);
}

/// Set file descriptors used
pub fn set_file_descriptors_used(count: i64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_FILE_DESCRIPTORS_USED, label, count);
}

/// Set file descriptor utilization ratio
pub fn set_file_descriptors_utilization(ratio: f64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_FILE_DESCRIPTORS_UTILIZATION, label, (ratio * 1000.0) as i64);
}

/// Record connection error
pub fn record_connection_error(error_type: &str, protocol: &str) {
    let label = ErrorTypeLabel {
        error_type: error_type.to_string(),
        protocol: protocol.to_string(),
    };
    counter_metric_inc!(NETWORK_CONNECTION_ERRORS, label);
}

/// Record codec error
pub fn record_codec_error(protocol: &str, error_type: &str) {
    let label = PacketErrorLabel {
        protocol: protocol.to_string(),
        error_type: error_type.to_string(),
    };
    counter_metric_inc!(NETWORK_CODEC_ERRORS, label);
}

/// Record write failure
pub fn record_write_failure(error_type: &str, protocol: &str) {
    let label = ErrorTypeLabel {
        error_type: error_type.to_string(),
        protocol: protocol.to_string(),
    };
    counter_metric_inc!(NETWORK_WRITE_FAILURES, label);
}

/// Record protocol violation
pub fn record_protocol_violation(protocol: &str, violation_type: &str) {
    let label = PacketErrorLabel {
        protocol: protocol.to_string(),
        error_type: violation_type.to_string(),
    };
    counter_metric_inc!(NETWORK_PROTOCOL_VIOLATIONS, label);
}

/// Record system call error
pub fn record_syscall_error(error_type: &str, protocol: &str) {
    let label = ErrorTypeLabel {
        error_type: error_type.to_string(),
        protocol: protocol.to_string(),
    };
    counter_metric_inc!(NETWORK_SYSCALL_ERRORS, label);
}

/// Record memory allocation failure
pub fn record_memory_allocation_failure() {
    let label = NoLabelSet;
    counter_metric_inc!(NETWORK_MEMORY_ALLOCATION_FAILURES, label);
}

/// Record thread creation failure
pub fn record_thread_creation_failure(thread_type: &str, network_type: &str) {
    let label = ThreadTypeLabel {
        thread_type: thread_type.to_string(),
        network_type: network_type.to_string(),
    };
    counter_metric_inc!(NETWORK_THREAD_CREATION_FAILURES, label);
}

/// Record MQTT connect attempt
pub fn record_mqtt_connect_attempt(result: &str) {
    let label = PacketErrorLabel {
        protocol: "mqtt".to_string(),
        error_type: result.to_string(),
    };
    counter_metric_inc!(MQTT_CONNECT_ATTEMPTS, label);
}

/// Set active MQTT subscriptions
pub fn set_mqtt_subscriptions_active(count: i64) {
    let label = NoLabelSet;
    gauge_metrics_set!(MQTT_SUBSCRIPTIONS_ACTIVE, label, count);
}

/// Set retained messages count
pub fn set_mqtt_retained_messages(count: i64) {
    let label = NoLabelSet;
    gauge_metrics_set!(MQTT_RETAINED_MESSAGES, label, count);
}

/// Record MQTT QoS distribution
pub fn record_mqtt_qos_distribution(qos: &str) {
    let label = PacketErrorLabel {
        protocol: "mqtt".to_string(),
        error_type: qos.to_string(),
    };
    counter_metric_inc!(MQTT_QOS_DISTRIBUTION, label);
}

/// Record Kafka produce request
pub fn record_kafka_produce_request() {
    let label = NoLabelSet;
    counter_metric_inc!(KAFKA_PRODUCE_REQUESTS, label);
}

/// Record Kafka consume request
pub fn record_kafka_consume_request() {
    let label = NoLabelSet;
    counter_metric_inc!(KAFKA_CONSUME_REQUESTS, label);
}

/// Record Kafka partition assignment
pub fn record_kafka_partition_assignment() {
    let label = NoLabelSet;
    counter_metric_inc!(KAFKA_PARTITION_ASSIGNMENTS, label);
}

/// Set server start time
pub fn set_server_start_time(timestamp: i64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_SERVER_START_TIME, label, timestamp);
}

/// Set server uptime
pub fn set_server_uptime(seconds: i64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_SERVER_UPTIME, label, seconds);
}

/// Set server status
pub fn set_server_status(status: &str, active: bool) {
    let label = ServerStatusLabel {
        status: status.to_string(),
    };
    gauge_metrics_set!(NETWORK_SERVER_STATUS, label, if active { 1 } else { 0 });
}

/// Set graceful shutdown progress
pub fn set_graceful_shutdown_progress(ratio: f64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_GRACEFUL_SHUTDOWN_PROGRESS, label, (ratio * 1000.0) as i64);
}

/// Set CPU usage ratio
pub fn set_cpu_usage_ratio(ratio: f64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_SERVER_CPU_USAGE, label, (ratio * 1000.0) as i64);
}

/// Set server load level
pub fn set_server_load_level(level: &str, active: bool) {
    let label = LoadLevelLabel {
        level: level.to_string(),
    };
    gauge_metrics_set!(NETWORK_SERVER_LOAD_LEVEL, label, if active { 1 } else { 0 });
}

/// Set backpressure status
pub fn set_backpressure_active(component: &str, active: bool) {
    let label = ComponentLabel {
        component: component.to_string(),
    };
    gauge_metrics_set!(NETWORK_BACKPRESSURE_ACTIVE, label, if active { 1 } else { 0 });
}

/// Set service availability ratio
pub fn set_service_availability(ratio: f64) {
    let label = NoLabelSet;
    gauge_metrics_set!(NETWORK_SERVICE_AVAILABILITY, label, (ratio * 1000.0) as i64);
}

/// Set request success rate
pub fn set_request_success_rate(protocol: &str, version: &str, rate: f64) {
    let label = NetworkProtocolVersionLabel {
        protocol: protocol.to_string(),
        version: version.to_string(),
    };
    gauge_metrics_set!(NETWORK_REQUEST_SUCCESS_RATE, label, (rate * 1000.0) as i64);
}

/// Set request latency percentile
pub fn set_request_latency_percentile(percentile: &str, protocol: &str, latency_ms: f64) {
    let label = PercentileLabel {
        percentile: percentile.to_string(),
        protocol: protocol.to_string(),
    };
    gauge_metrics_set!(NETWORK_REQUEST_LATENCY_PERCENTILE, label, latency_ms as i64);
}

/// Set network throughput
pub fn set_network_throughput(direction: &str, protocol: &str, bytes_per_second: f64) {
    let label = DirectionLabel {
        direction: direction.to_string(),
        protocol: protocol.to_string(),
    };
    gauge_metrics_set!(NETWORK_THROUGHPUT, label, bytes_per_second as i64);
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_metrics() {
        record_connection_established("tcp");
        record_connection_closed("tcp", "client_close");
        record_connection_duration("tcp", 5000.0);
        record_connection_establishment_duration("tcp", 100.0);
        record_tls_handshake_duration("tls", 200.0);
    }

    #[test]
    fn test_packet_metrics() {
        record_packet_received("mqtt", "5", 1024.0);
        record_packet_sent("mqtt", "5", 512.0);
        record_packet_error("mqtt", "decode");
        record_packet_dropped("mqtt", "queue_full");
    }

    #[test]
    fn test_performance_metrics() {
        record_request_duration("mqtt", "publish", 50.0);
        record_request_queue_duration("request_main", "tcp", 10.0);
        record_request_handler_duration("mqtt", "publish", 30.0);
        record_response_send_duration("tcp", 5.0);
    }

    #[test]
    fn test_queue_metrics() {
        set_queue_size("request_main", "tcp", 100);
        record_queue_full("request_main", "tcp");
        set_queue_processing_rate("request_main", "tcp", 1000.0);
    }

    #[test]
    fn test_thread_metrics() {
        set_thread_count("acceptor", "tcp", 4);
        set_active_thread_count("acceptor", "tcp", 3);
        set_thread_utilization("acceptor", "tcp", 0.75);
        record_thread_lifecycle("acceptor", "tcp");
    }

    #[test]
    fn test_memory_metrics() {
        set_connection_manager_memory(1024 * 1024);
        set_queue_memory("request_main", "tcp", 512 * 1024);
        set_codec_buffer_memory("mqtt", "5", 64 * 1024);
        set_write_buffer_size("tcp", 32 * 1024);
        set_file_descriptors_used(1000);
        set_file_descriptors_utilization(0.5);
    }

    #[test]
    fn test_error_metrics() {
        record_connection_error("timeout", "tcp");
        record_codec_error("mqtt", "decode");
        record_write_failure("connection_closed", "tcp");
        record_protocol_violation("mqtt", "invalid_packet");
        record_syscall_error("accept", "tcp");
        record_memory_allocation_failure();
        record_thread_creation_failure("acceptor", "tcp");
    }

    #[test]
    fn test_business_metrics() {
        record_mqtt_connect_attempt("success");
        set_mqtt_subscriptions_active(1000);
        set_mqtt_retained_messages(500);
        record_mqtt_qos_distribution("1");
        record_kafka_produce_request();
        record_kafka_consume_request();
        record_kafka_partition_assignment();
    }

    #[test]
    fn test_health_metrics() {
        set_server_start_time(1640995200);
        set_server_uptime(3600);
        set_server_status("running", true);
        set_graceful_shutdown_progress(0.5);
        set_cpu_usage_ratio(0.8);
        set_server_load_level("medium", true);
        set_backpressure_active("acceptor", false);
    }

    #[test]
    fn test_sla_metrics() {
        set_service_availability(0.999);
        set_request_success_rate("mqtt", "5", 0.995);
        set_request_latency_percentile("99", "mqtt", 100.0);
        set_network_throughput("inbound", "tcp", 1024000.0);
    }
}