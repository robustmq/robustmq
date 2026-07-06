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

// File-segment scroll and distribution drill.
//
// Requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml). Marked
// `#[ignore]`; run with:
//   cargo test -p robustmq-test file_segment_scroll_and_distribution -- --ignored --nocapture
//
// Writes ~33 MB of data into a shard (EngineSegment, max_segment_size = 10 MB,
// 3 replicas). Scroll triggers when an offset hits a 10 000 multiple AND the
// segment file exceeds 90 % of max_segment_size. With ~1.1 KB records, that
// condition is satisfied at the first 10 000-offset checkpoint (~11 MB > 9 MB).
//
// After writing, the test asserts:
//   - 4 segments exist (seq 0-3); seqs 0-2 are SealUp, seq 3 is Write.
//   - Every segment has replicas on all 3 nodes.
//   - Sealed segments have continuous, gap-free offsets:
//       seg[i].end_offset + 1 == seg[i+1].start_offset, seg[0].start_offset == 0
//   - Sealed segments have monotonically increasing timestamps:
//       start_ts > 0, end_ts > 0, start_ts <= end_ts per segment,
//       seg[i].start_ts <= seg[i+1].start_ts across segments.

#[cfg(test)]
mod tests {
    use crate::mqtt::protocol::common::create_test_env;
    use admin_server::client::AdminHttpClient;
    use admin_server::engine::segment::{SegmentListReq, SegmentListResp, SegmentListRespRaw};
    use admin_server::engine::shard::{ShardCreateReq, ShardDeleteReq};
    use broker_core::cache::NodeCacheManager;
    use bytes::Bytes;
    use common_base::http_response::AdminServerResponse;
    use common_base::utils::serialize;
    use common_base::uuid::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::meta::node::BrokerNode;
    use metadata_struct::storage::segment::SegmentStatus;
    use protocol::storage::protocol::WriteReqBody;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::core::cache::StorageCacheManager;
    use tokio::time::sleep;

    const NODES: [u64; 3] = [1, 2, 3];
    const SEGMENT_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
    const RECORD_PAYLOAD: usize = 1100; // ~1.1 KB → 10 000 records ≈ 11 MB > 90 % of 10 MB
    const BATCH_RECORDS: u64 = 1000; // ~1.1 MB per batch
    const TARGET_SEGMENTS: usize = 4;

    // ── address / client helpers ──────────────────────────────────────────────

    fn engine_addr(node_id: u64) -> String {
        match node_id {
            1 => "127.0.0.1:1779",
            2 => "127.0.0.1:2779",
            3 => "127.0.0.1:3779",
            other => panic!("unknown node id {other}"),
        }
        .to_string()
    }

    fn build_write_client() -> Arc<ClientConnectionManager> {
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        for node_id in NODES {
            broker_cache.add_node(BrokerNode {
                node_id,
                engine_addr: engine_addr(node_id),
                ..Default::default()
            });
        }
        let cache = Arc::new(StorageCacheManager::new(broker_cache));
        Arc::new(ClientConnectionManager::new(cache, 2))
    }

    // ── admin API helpers ─────────────────────────────────────────────────────

    async fn get_segment_list(
        admin: &AdminHttpClient,
        shard_name: &str,
    ) -> Vec<SegmentListRespRaw> {
        let raw = admin
            .get_segment_list(&SegmentListReq {
                shard_name: shard_name.to_string(),
            })
            .await
            .unwrap();
        let resp: AdminServerResponse<SegmentListResp> = serde_json::from_str(&raw).unwrap();
        let mut list = resp.data.segment_list;
        list.sort_by_key(|r| r.segment.segment_seq);
        list
    }

    /// Poll until `want_total` segments exist and `want_sealed` of them are SealUp (60 s timeout).
    async fn wait_segments(
        admin: &AdminHttpClient,
        shard_name: &str,
        want_total: usize,
        want_sealed: usize,
    ) -> Vec<SegmentListRespRaw> {
        let start = Instant::now();
        loop {
            let list = get_segment_list(admin, shard_name).await;
            let sealed = list
                .iter()
                .filter(|r| matches!(r.segment.status, SegmentStatus::SealUp))
                .count();
            if list.len() >= want_total && sealed >= want_sealed {
                return list;
            }
            assert!(
                start.elapsed() < Duration::from_secs(60),
                "timeout waiting for segments: have {}/{} total, {}/{} sealed",
                list.len(),
                want_total,
                sealed,
                want_sealed,
            );
            sleep(Duration::from_secs(1)).await;
        }
    }

    // ── write helper ──────────────────────────────────────────────────────────

    /// Write `n` records of ~RECORD_PAYLOAD bytes each to `leader`, acks=1.
    async fn write_batch(
        writer: &Arc<ClientConnectionManager>,
        leader: u64,
        shard_name: &str,
        n: u64,
    ) {
        let payload = Bytes::from(vec![b'x'; RECORD_PAYLOAD]);
        let messages: Vec<_> = (0..n)
            .map(|_| {
                let rec = AdapterWriteRecord::new("", payload.clone());
                serialize::serialize(&rec).unwrap()
            })
            .collect();
        let mut body = WriteReqBody::new(shard_name.to_string(), messages);
        body.acks = 1;
        writer
            .send_write_body(leader, body)
            .await
            .unwrap_or_else(|e| panic!("write to n{leader} failed: {e}"));
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Drill — file segment scroll + replica distribution
    // ═════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore = "requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml)"]
    async fn file_segment_scroll_and_distribution() {
        // ── 1. create shard ───────────────────────────────────────────────────
        let admin = create_test_env().await;
        let shard_name = unique_id();
        let config = format!(
            r#"{{"replica_num":3,"max_segment_size":{},"retention_sec":86400,"storage_type":"EngineSegment"}}"#,
            SEGMENT_SIZE
        );
        let create_resp: AdminServerResponse<serde_json::Value> = serde_json::from_str(
            &admin
                .create_shard(&ShardCreateReq {
                    shard_name: shard_name.clone(),
                    topic_name: None,
                    desc: None,
                    config,
                })
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            create_resp.code, 0,
            "create_shard failed: {:?}",
            create_resp.error
        );
        sleep(Duration::from_secs(10)).await;

        // ── 2. write until 4 segments appear ─────────────────────────────────
        // Each batch ≈ 1.1 MB. Scroll check fires at every 10 000-offset multiple;
        // with 1.1 KB records, 10 000 records ≈ 11 MB > 9 MB (90 % of 10 MB) so
        // scroll triggers at the first checkpoint. ~30 batches ≈ 33 MB → 4 segments.
        let writer = build_write_client();

        // Determine initial leader from segment 0.
        let mut segments = get_segment_list(&admin, &shard_name).await;
        assert!(!segments.is_empty(), "no segments after shard creation");
        let mut leader = segments.last().unwrap().segment.leader;
        assert_ne!(leader, 0, "leader must be assigned before writing");
        println!("initial leader: n{leader}");

        let mut total_batches: u64 = 0;
        loop {
            write_batch(&writer, leader, &shard_name, BATCH_RECORDS).await;
            total_batches += 1;

            segments = get_segment_list(&admin, &shard_name).await;
            let sealed_count = segments
                .iter()
                .filter(|r| matches!(r.segment.status, SegmentStatus::SealUp))
                .count();

            println!(
                "batch {:>3}: {} segments ({} sealed) | ~{} MB written",
                total_batches,
                segments.len(),
                sealed_count,
                total_batches * BATCH_RECORDS * RECORD_PAYLOAD as u64 / 1024 / 1024,
            );

            // If the active segment changed leader, follow it.
            if let Some(active) = segments.last() {
                let new_leader = active.segment.leader;
                if new_leader != 0 && new_leader != leader {
                    println!("  active segment leader: n{leader} → n{new_leader}");
                    leader = new_leader;
                }
            }

            if segments.len() >= TARGET_SEGMENTS {
                break;
            }

            assert!(
                total_batches < 60,
                "wrote 60 batches (~66 MB) but still <{TARGET_SEGMENTS} segments — scroll not triggering"
            );
        }

        // ── 3. wait for segs 0-2 to fully seal ───────────────────────────────
        let segments =
            wait_segments(&admin, &shard_name, TARGET_SEGMENTS, TARGET_SEGMENTS - 1).await;

        // ── 4. print summary ──────────────────────────────────────────────────
        println!("\n=== segment summary ===");
        for raw in &segments {
            let s = &raw.segment;
            let m = raw.segment_meta.as_ref();
            println!(
                "  seg {seq}: status={status:?}  leader=n{leader}  \
                 replicas={replicas:?}  \
                 start_offset={so}  end_offset={eo}  \
                 start_ts={st}  end_ts={et}",
                seq = s.segment_seq,
                status = s.status,
                leader = s.leader,
                replicas = s.replicas.iter().map(|r| r.node_id).collect::<Vec<_>>(),
                so = m.map_or(-1, |x| x.start_offset),
                eo = m.map_or(-1, |x| x.end_offset),
                st = m.map_or(-1, |x| x.start_timestamp),
                et = m.map_or(-1, |x| x.end_timestamp),
            );
        }

        // ── 5. assertions ─────────────────────────────────────────────────────

        let total = segments.len();
        assert!(total >= TARGET_SEGMENTS);

        // (a) status: seqs 0..total-2 must be SealUp; last one is Write
        for raw in &segments {
            let seq = raw.segment.segment_seq;
            if seq < total as u32 - 1 {
                assert!(
                    matches!(raw.segment.status, SegmentStatus::SealUp),
                    "seg {seq} expected SealUp, got {:?}",
                    raw.segment.status
                );
            }
        }
        assert!(
            matches!(
                segments.last().unwrap().segment.status,
                SegmentStatus::Write
            ),
            "active seg expected Write, got {:?}",
            segments.last().unwrap().segment.status
        );

        // (b) every segment must have replicas on all 3 nodes
        for raw in &segments {
            let seq = raw.segment.segment_seq;
            let mut node_ids: Vec<u64> = raw.segment.replicas.iter().map(|r| r.node_id).collect();
            node_ids.sort_unstable();
            assert_eq!(
                node_ids,
                vec![1, 2, 3],
                "seg {seq} replicas must cover all 3 nodes, got {node_ids:?}"
            );
        }

        // (c) leader is always a valid node
        for raw in &segments {
            let seq = raw.segment.segment_seq;
            assert!(
                NODES.contains(&raw.segment.leader),
                "seg {seq} leader n{} not in {{1,2,3}}",
                raw.segment.leader
            );
        }

        // (d) sealed segment offset chain: continuous, no gaps
        let sealed: Vec<_> = segments
            .iter()
            .filter(|r| matches!(r.segment.status, SegmentStatus::SealUp))
            .collect();

        let first_meta = sealed[0]
            .segment_meta
            .as_ref()
            .expect("sealed seg 0 must have metadata");
        assert_eq!(
            first_meta.start_offset, 0,
            "seg 0 start_offset must be 0, got {}",
            first_meta.start_offset
        );

        for i in 0..sealed.len() - 1 {
            let cur = sealed[i].segment_meta.as_ref().unwrap_or_else(|| {
                panic!(
                    "sealed seg {} must have metadata",
                    sealed[i].segment.segment_seq
                )
            });
            let next = sealed[i + 1].segment_meta.as_ref().unwrap_or_else(|| {
                panic!(
                    "sealed seg {} must have metadata",
                    sealed[i + 1].segment.segment_seq
                )
            });
            assert!(
                cur.end_offset > 0,
                "seg {} end_offset not set (got {})",
                sealed[i].segment.segment_seq,
                cur.end_offset
            );
            assert_eq!(
                cur.end_offset + 1,
                next.start_offset,
                "offset gap: seg {} end={} but seg {} start={}",
                sealed[i].segment.segment_seq,
                cur.end_offset,
                sealed[i + 1].segment.segment_seq,
                next.start_offset
            );
        }

        // (e) sealed segment timestamps: set, internally consistent, monotonically increasing
        for raw in &sealed {
            let seq = raw.segment.segment_seq;
            let meta = raw.segment_meta.as_ref().unwrap();
            assert!(
                meta.start_timestamp > 0,
                "seg {seq} start_timestamp must be > 0"
            );
            assert!(
                meta.end_timestamp > 0,
                "seg {seq} end_timestamp must be > 0 (segment is sealed)"
            );
            assert!(
                meta.start_timestamp <= meta.end_timestamp,
                "seg {seq} start_ts ({}) > end_ts ({})",
                meta.start_timestamp,
                meta.end_timestamp
            );
        }
        for i in 0..sealed.len() - 1 {
            let cur = sealed[i].segment_meta.as_ref().unwrap();
            let next = sealed[i + 1].segment_meta.as_ref().unwrap();
            assert!(
                cur.start_timestamp <= next.start_timestamp,
                "seg {} start_ts ({}) > seg {} start_ts ({})",
                sealed[i].segment.segment_seq,
                cur.start_timestamp,
                sealed[i + 1].segment.segment_seq,
                next.start_timestamp
            );
        }

        // (f) leader distribution (informational — not strictly required to be uniform)
        let leader_dist: HashSet<u64> = segments.iter().map(|r| r.segment.leader).collect();
        println!(
            "\n=== leader distribution: {} unique leader(s) across {} segments: {:?} ===",
            leader_dist.len(),
            total,
            leader_dist
        );

        // ── 6. cleanup ────────────────────────────────────────────────────────
        let _ = admin
            .delete_shard(&ShardDeleteReq {
                shard_name: shard_name.clone(),
            })
            .await;

        println!(
            "\nPASS: {total} segments, offset chain continuous, \
             timestamps monotonic, all replicas on {{1,2,3}}"
        );
    }
}
