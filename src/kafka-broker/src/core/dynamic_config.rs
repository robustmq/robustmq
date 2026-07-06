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

//! Registry of Kafka's dynamic configuration keys (the ones settable via
//! `AlterConfigsReq`/`IncrementalAlterConfigsReq`), and which of them
//! RobustMQ currently has a concrete field to back.
//!
//! Not implementation — `kafka/config.rs`'s `process_alter_configs`/
//! `process_incremental_alter_configs` are still stubs. This module exists
//! so that work can validate/reject unknown config names and know which
//! Rust field a supported name maps to, instead of re-deriving the list of
//! ~30 topic configs and ~40+ broker configs from memory or Kafka docs
//! every time.

/// Kafka's `ConfigResource.Type` wire values (not modeled by the
/// `kafka-protocol` crate as an enum — `AlterConfigsRequest`/
/// `IncrementalAlterConfigsRequest` carry this as a raw `i8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigResourceType {
    Topic,
    Broker,
    /// A "resource name" for this type is a logger name (e.g.
    /// `kafka.controller`), not a fixed config key — there is no
    /// `BROKER_LOGGER_CONFIGS` list below because the valid "keys" are
    /// whatever loggers exist at runtime, and the "value" is a log level.
    BrokerLogger,
}

impl ConfigResourceType {
    pub fn from_wire(value: i8) -> Option<Self> {
        match value {
            2 => Some(Self::Topic),
            4 => Some(Self::Broker),
            8 => Some(Self::BrokerLogger),
            _ => None,
        }
    }
}

/// One Kafka dynamic config key.
pub struct DynamicConfigKey {
    pub name: &'static str,
    pub default: &'static str,
    pub description: &'static str,
    /// The `EngineShardConfig`/`Topic` field this maps to today, if any.
    /// `None` means the config is recognized (so it shouldn't be rejected
    /// as unknown) but RobustMQ has nothing to apply it to yet.
    pub robustmq_field: Option<&'static str>,
}

/// Topic-level dynamic configs (`ConfigResourceType::Topic`), i.e. what
/// `kafka-configs.sh --entity-type topics --alter` operates on. Source of
/// truth is Kafka's `org.apache.kafka.common.config.TopicConfig`.
///
/// Only three currently map to a real RobustMQ field
/// (`metadata_struct::storage::shard::EngineShardConfig`): `retention.ms`,
/// `segment.bytes`, `min.insync.replicas`. Everything else is listed so a
/// future `process_alter_configs` can tell "unsupported but valid Kafka
/// config" apart from "not a real Kafka config at all" (`InvalidConfig`
/// vs `InvalidRequest`), and report the former as a no-op rather than an
/// error.
pub const TOPIC_CONFIGS: &[DynamicConfigKey] = &[
    DynamicConfigKey {
        name: "cleanup.policy",
        default: "delete",
        description: "Whether old segments are dropped (delete), compacted (compact), or both.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "compression.type",
        default: "producer",
        description: "Compression codec applied to a topic's stored records.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "delete.retention.ms",
        default: "86400000",
        description: "How long compacted-topic delete tombstones are retained.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "file.delete.delay.ms",
        default: "60000",
        description: "Delay before a deleted segment's file is removed from disk.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "flush.messages",
        default: "9223372036854775807",
        description: "Number of messages accumulated before a forced fsync.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "flush.ms",
        default: "9223372036854775807",
        description: "Max time before a forced fsync of accumulated messages.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "follower.replication.throttled.replicas",
        default: "",
        description: "Replicas whose follower-side replication traffic is throttled.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "index.interval.bytes",
        default: "4096",
        description: "Byte interval at which an index entry is added to the offset index.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "leader.replication.throttled.replicas",
        default: "",
        description: "Replicas whose leader-side replication traffic is throttled.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "max.compaction.lag.ms",
        default: "9223372036854775807",
        description: "Max time a message can remain uncompacted in a compacted topic.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "max.message.bytes",
        default: "1048588",
        description: "Largest record batch size the broker accepts for this topic.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "message.timestamp.type",
        default: "CreateTime",
        description: "Whether record timestamps are producer CreateTime or broker LogAppendTime.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "message.timestamp.before.max.ms",
        default: "9223372036854775807",
        description: "How far in the past a record timestamp may be vs. broker time.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "message.timestamp.after.max.ms",
        default: "9223372036854775807",
        description: "How far in the future a record timestamp may be vs. broker time.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "min.cleanable.dirty.ratio",
        default: "0.5",
        description: "Ratio of dirty-to-total log bytes that triggers compaction.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "min.compaction.lag.ms",
        default: "0",
        description: "Minimum time a message must remain uncompacted.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "min.insync.replicas",
        default: "1",
        description: "Minimum in-sync replicas required for an acks=all write to succeed.",
        robustmq_field: Some("EngineShardConfig::min_in_sync_replicas"),
    },
    DynamicConfigKey {
        name: "preallocate",
        default: "false",
        description: "Whether new segment files are preallocated to their max size on disk.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "retention.bytes",
        default: "-1",
        description: "Max total size of a partition's log before old segments are dropped.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "retention.ms",
        default: "604800000",
        description: "Max age of a record before it becomes eligible for deletion.",
        // Unit mismatch to account for when implementing: Kafka is
        // milliseconds, RobustMQ's field is seconds.
        robustmq_field: Some("EngineShardConfig::retention_sec"),
    },
    DynamicConfigKey {
        name: "segment.bytes",
        default: "1073741824",
        description: "Max size of a single log segment file.",
        robustmq_field: Some("EngineShardConfig::max_segment_size"),
    },
    DynamicConfigKey {
        name: "segment.index.bytes",
        default: "10485760",
        description: "Max size of a segment's offset index file.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "segment.jitter.ms",
        default: "0",
        description: "Random jitter subtracted from segment.ms to avoid thundering-herd rolls.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "segment.ms",
        default: "604800000",
        description: "Max time before a segment is force-rolled even if not full.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "unclean.leader.election.enable",
        default: "false",
        description: "Whether an out-of-ISR replica may be elected leader, risking data loss.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "message.downconversion.enable",
        default: "true",
        description: "Whether the broker downconverts message format for older-version consumers.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "remote.storage.enable",
        default: "false",
        description: "Whether tiered storage is enabled for this topic (KIP-405).",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "local.retention.ms",
        default: "-2",
        description: "Retention on local disk before a segment is eligible to move to remote tier.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "local.retention.bytes",
        default: "-2",
        description:
            "Local-disk size threshold before a segment is eligible to move to remote tier.",
        robustmq_field: None,
    },
];

/// Broker-level dynamic configs (`ConfigResourceType::Broker`), i.e. what
/// `kafka-configs.sh --entity-type brokers --alter` operates on. RobustMQ
/// has no dynamic-broker-config system today (its own `common/config`
/// values are read once at startup, not hot-reloadable via a Kafka-style
/// admin call), so every entry here is currently unsupported — listed so a
/// future implementation can at least recognize valid names.
pub const BROKER_CONFIGS: &[DynamicConfigKey] = &[
    DynamicConfigKey {
        name: "background.threads",
        default: "10",
        description: "Threads for background housekeeping (log cleanup, deletion, etc).",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "log.cleaner.threads",
        default: "1",
        description: "Threads dedicated to log compaction.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "log.retention.bytes",
        default: "-1",
        description: "Cluster-wide default retention.bytes for topics that don't override it.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "log.retention.ms",
        default: "604800000",
        description: "Cluster-wide default retention.ms for topics that don't override it.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "log.segment.bytes",
        default: "1073741824",
        description: "Cluster-wide default segment.bytes for topics that don't override it.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "log.retention.check.interval.ms",
        default: "300000",
        description: "How often the broker checks for logs eligible for deletion.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "max.connections",
        default: "2147483647",
        description: "Max simultaneous connections accepted per broker.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "max.connections.per.ip",
        default: "2147483647",
        description: "Max simultaneous connections accepted per source IP.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "message.max.bytes",
        default: "1048588",
        description: "Cluster-wide default max.message.bytes for topics that don't override it.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "min.insync.replicas",
        default: "1",
        description: "Cluster-wide default min.insync.replicas for topics that don't override it.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "num.io.threads",
        default: "8",
        description: "Threads the broker uses for disk I/O.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "num.network.threads",
        default: "3",
        description: "Threads the broker uses for network request handling.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "num.replica.fetchers",
        default: "1",
        description: "Threads a follower uses to fetch from the partition leader.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "unclean.leader.election.enable",
        default: "false",
        description:
            "Cluster-wide default unclean-election policy for topics that don't override it.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "leader.replication.throttled.rate",
        default: "9223372036854775807",
        description: "Byte-rate cap on leader-side throttled replication traffic.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "follower.replication.throttled.rate",
        default: "9223372036854775807",
        description: "Byte-rate cap on follower-side throttled replication traffic.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "ssl.keystore.location",
        default: "",
        description: "Path to the SSL keystore; reconfigurable without a broker restart.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "ssl.keystore.password",
        default: "",
        description: "Password for the SSL keystore; reconfigurable without a broker restart.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "ssl.truststore.location",
        default: "",
        description: "Path to the SSL truststore; reconfigurable without a broker restart.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "sasl.jaas.config",
        default: "",
        description: "Per-listener JAAS login config; reconfigurable without a broker restart.",
        robustmq_field: None,
    },
    DynamicConfigKey {
        name: "advertised.listeners",
        default: "",
        description: "Listener addresses advertised to clients.",
        robustmq_field: None,
    },
];

/// Look up a topic-level config by name.
pub fn find_topic_config(name: &str) -> Option<&'static DynamicConfigKey> {
    TOPIC_CONFIGS.iter().find(|c| c.name == name)
}

/// Look up a broker-level config by name.
pub fn find_broker_config(name: &str) -> Option<&'static DynamicConfigKey> {
    BROKER_CONFIGS.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_wire_maps_known_resource_types() {
        assert_eq!(
            ConfigResourceType::from_wire(2),
            Some(ConfigResourceType::Topic)
        );
        assert_eq!(
            ConfigResourceType::from_wire(4),
            Some(ConfigResourceType::Broker)
        );
        assert_eq!(
            ConfigResourceType::from_wire(8),
            Some(ConfigResourceType::BrokerLogger)
        );
        assert_eq!(ConfigResourceType::from_wire(0), None);
    }

    #[test]
    fn find_topic_config_distinguishes_supported_from_recognized_only() {
        assert_eq!(
            find_topic_config("retention.ms").unwrap().robustmq_field,
            Some("EngineShardConfig::retention_sec")
        );
        assert_eq!(
            find_topic_config("cleanup.policy").unwrap().robustmq_field,
            None
        );
        assert!(find_topic_config("not.a.real.config").is_none());
    }
}
