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

// ISR replica-sync integration test.
//
// Requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml). Marked
// `#[ignore]`; run with `cargo test -p tests three_replica_isr_sync -- --ignored --nocapture`.
//
// Scenario: create a 3-replica shard, write data continuously to the leader, then
// poll the segment-detail admin API and assert that all three replicas join the
// ISR and their LEOs converge to the leader's (the high watermark advances to the
// committed offset). Note: ISR leader/follower replication currently applies to
// EngineRocksDB / EngineMemory storage, NOT EngineSegment — so this uses RocksDB.

#[cfg(test)]
mod tests {
    use crate::mqtt::protocol::common::create_test_env;
    use admin_server::client::AdminHttpClient;
    use admin_server::engine::segment::{
        SegmentDetailReq, SegmentDetailResp, SegmentListReq, SegmentListResp,
    };
    use admin_server::engine::shard::{ShardCreateReq, ShardDeleteReq, ShardListReq, ShardListRow};
    use broker_core::cache::NodeCacheManager;
    use bytes::Bytes;
    use common_base::http_response::AdminServerResponse;
    use common_base::utils::serialize;
    use common_base::uuid::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::meta::node::BrokerNode;
    use protocol::storage::codec::StorageEnginePacket;
    use protocol::storage::protocol::{WriteReq, WriteReqBody};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::core::cache::StorageCacheManager;
    use tokio::time::sleep;

    const NODES: [u64; 3] = [1, 2, 3];
    const REPLICA_NUM: usize = 3;

    /// Storage-engine TCP port of each dev-cluster node (config/cluster/server-N.toml).
    fn engine_addr(node_id: u64) -> String {
        match node_id {
            1 => "127.0.0.1:1779",
            2 => "127.0.0.1:2779",
            3 => "127.0.0.1:3779",
            other => panic!("unknown node id {other}"),
        }
        .to_string()
    }

    /// A write client that knows every node's engine address, so it can send the
    /// write directly to whichever node is the segment leader.
    fn build_write_client() -> Arc<ClientConnectionManager> {
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        for node_id in NODES {
            broker_cache.add_node(BrokerNode {
                node_id,
                engine_addr: engine_addr(node_id),
                ..Default::default()
            });
        }
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        Arc::new(ClientConnectionManager::new(cache_manager, 2))
    }

    async fn get_shard(admin: &AdminHttpClient, shard_name: &str) -> ShardListRow {
        let resp = admin
            .get_shard_list::<_, Vec<ShardListRow>>(&ShardListReq {
                shard_name: Some(shard_name.to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(
            resp.data.len(),
            1,
            "expected exactly one shard '{shard_name}'"
        );
        resp.data.into_iter().next().unwrap()
    }

    async fn active_segment_seq(admin: &AdminHttpClient, shard_name: &str) -> u32 {
        let resp_str = admin
            .get_segment_list(&SegmentListReq {
                shard_name: shard_name.to_string(),
            })
            .await
            .unwrap();
        let resp: AdminServerResponse<SegmentListResp> = serde_json::from_str(&resp_str).unwrap();
        let list = resp.data.segment_list;
        assert!(!list.is_empty(), "no segments for shard '{shard_name}'");
        // Active (writeable) segment = highest seq; a fresh shard has only seq 0.
        list.iter().map(|s| s.segment.segment_seq).max().unwrap()
    }

    async fn segment_detail(
        admin: &AdminHttpClient,
        shard_name: &str,
        segment_seq: u32,
    ) -> SegmentDetailResp {
        let req = SegmentDetailReq {
            shard_name: shard_name.to_string(),
            segment_seq,
        };
        admin
            .get_segment_detail::<_, SegmentDetailResp>(&req)
            .await
            .unwrap()
    }

    fn replicas_line(detail: &SegmentDetailResp) -> String {
        detail
            .replicas
            .iter()
            .map(|r| {
                format!(
                    "n{}(leo={},hw={},isr={},avail={})",
                    r.node_id, r.leo, r.high_watermark, r.in_isr, r.available
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[tokio::test]
    #[ignore = "requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml)"]
    async fn three_replica_isr_sync() {
        let admin = create_test_env().await; // http://127.0.0.1:58080
        let shard_name = unique_id();

        // 1. Create a 3-replica RocksDB shard (ISR replication applies to RocksDB/Memory).
        let config = r#"{"replica_num":3,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#.to_string();
        let create = admin
            .create_shard(&ShardCreateReq {
                shard_name: shard_name.clone(),
                topic_name: None,
                desc: None,
                config,
            })
            .await
            .unwrap();
        let create_resp: AdminServerResponse<serde_json::Value> =
            serde_json::from_str(&create).unwrap();
        assert_eq!(
            create_resp.code, 0,
            "create_shard failed: {:?}",
            create_resp.error
        );

        // Wait for the shard + segment 0 to be provisioned and the leader elected.
        sleep(Duration::from_secs(10)).await;

        // 2. Get shard: confirm it is genuinely a 3-replica shard and inspect its
        //    offsets (start / end / high watermark).
        let shard = get_shard(&admin, &shard_name).await;
        assert_eq!(
            shard.shard_info.config.replica_num, REPLICA_NUM as u32,
            "shard should be configured with {REPLICA_NUM} replicas"
        );
        println!(
            "get_shard '{shard_name}': replica_num={} start_offset={} end_offset={} high_watermark={}",
            shard.shard_info.config.replica_num,
            shard.shard_info.offset.start_offset,
            shard.shard_info.offset.end_offset,
            shard.shard_info.offset.high_watermark,
        );

        // 3. Discover the active segment and its leader.
        let segment_seq = active_segment_seq(&admin, &shard_name).await;
        let detail = segment_detail(&admin, &shard_name, segment_seq).await;
        assert_eq!(
            detail.segment.replicas.len(),
            REPLICA_NUM,
            "expected {REPLICA_NUM} replicas, got {:?}",
            detail.segment.replicas
        );
        let leader = detail.segment.leader;
        assert!(leader != 0, "no leader elected for segment");
        println!(
            "shard '{shard_name}' seg {segment_seq}: leader=n{leader} replicas={:?} isr={:?}",
            detail
                .segment
                .replicas
                .iter()
                .map(|r| r.node_id)
                .collect::<Vec<_>>(),
            detail.segment.isr
        );

        // 4. Write data continuously to the leader (acks=1 default; followers
        //    replicate asynchronously, which we then observe via segment detail).
        let writer = build_write_client();
        let total_batches = 10u64;
        let per_batch = 50u64;
        let expected_leo = total_batches * per_batch;
        for batch in 0..total_batches {
            let mut messages = Vec::with_capacity(per_batch as usize);
            for i in 0..per_batch {
                let n = batch * per_batch + i;
                let record = AdapterWriteRecord::new("", Bytes::from(format!("isr-data-{n}")));
                messages.push(serialize::serialize(&record).unwrap());
            }
            let write_req = WriteReq::new(WriteReqBody::new(shard_name.clone(), messages));
            let resp = writer
                .write_send(leader, StorageEnginePacket::WriteReq(write_req))
                .await
                .unwrap();
            match resp {
                StorageEnginePacket::WriteResp(r) => {
                    if let Some(e) = r.header.error {
                        panic!(
                            "write batch {batch} failed: code={}, msg={}",
                            e.code, e.error
                        );
                    }
                }
                other => panic!("expected WriteResp, got {other:?}"),
            }
            sleep(Duration::from_millis(200)).await;
        }
        println!("wrote {expected_leo} records to leader n{leader}");

        // 5. Poll segment detail until all replicas join the ISR and catch up.
        let deadline = Duration::from_secs(30);
        let start = Instant::now();
        loop {
            let d = segment_detail(&admin, &shard_name, segment_seq).await;
            let leader_leo = d
                .replicas
                .iter()
                .find(|r| r.node_id == leader)
                .map(|r| r.leo)
                .unwrap_or(0);
            let all_in_isr =
                d.segment.isr.len() == REPLICA_NUM && d.replicas.iter().all(|r| r.in_isr);
            // Converged = every replica available, caught up to the leader's LEO,
            // AND its high watermark has propagated up to that committed offset.
            let all_caught = d.replicas.len() == REPLICA_NUM
                && d.replicas
                    .iter()
                    .all(|r| r.available && r.leo == leader_leo && r.high_watermark == leader_leo);

            println!(
                "[{:?}] leader_leo={leader_leo} isr={} | {}",
                start.elapsed(),
                d.segment.isr.len(),
                replicas_line(&d)
            );

            if all_in_isr && all_caught && leader_leo >= expected_leo {
                println!("ISR converged: 3 replicas all at LEO=HW={leader_leo}");
                break;
            }
            if start.elapsed() > deadline {
                panic!(
                    "ISR did not converge within {deadline:?}: isr={:?} replicas={:#?}",
                    d.segment.isr, d.replicas
                );
            }
            sleep(Duration::from_millis(500)).await;
        }

        // 6. Final assertions. Re-read the shard to show offsets / HW advanced.
        let final_shard = get_shard(&admin, &shard_name).await;
        println!(
            "get_shard '{shard_name}' after writes: start_offset={} end_offset={} high_watermark={}",
            final_shard.shard_info.offset.start_offset,
            final_shard.shard_info.offset.end_offset,
            final_shard.shard_info.offset.high_watermark,
        );
        // get_shard is served by the admin-entry node (which may be a follower);
        // its HW must reflect the committed offset now that followers track HW.
        assert_eq!(
            final_shard.shard_info.offset.high_watermark, expected_leo,
            "get_shard HW {} should equal committed LEO {expected_leo}",
            final_shard.shard_info.offset.high_watermark
        );
        assert_eq!(
            final_shard.shard_info.offset.end_offset,
            expected_leo - 1,
            "get_shard end_offset should be {}",
            expected_leo - 1
        );

        let final_detail = segment_detail(&admin, &shard_name, segment_seq).await;
        let leader_replica = final_detail
            .replicas
            .iter()
            .find(|r| r.node_id == leader)
            .expect("leader replica present");
        let leader_leo = leader_replica.leo;
        let leader_hw = leader_replica.high_watermark;

        assert_eq!(
            final_detail.segment.isr.len(),
            REPLICA_NUM,
            "all {REPLICA_NUM} replicas must be in ISR, got {:?}",
            final_detail.segment.isr
        );
        for r in &final_detail.replicas {
            assert!(r.in_isr, "replica n{} not in ISR", r.node_id);
            assert!(r.available, "replica n{} not available", r.node_id);
            assert_eq!(
                r.leo, leader_leo,
                "replica n{} LEO {} != leader LEO {}",
                r.node_id, r.leo, leader_leo
            );
            // Every replica (leader and followers) must have advanced its HW to
            // the committed offset — followers learn it from the fetch response.
            assert_eq!(
                r.high_watermark, leader_leo,
                "replica n{} HW {} != committed LEO {}",
                r.node_id, r.high_watermark, leader_leo
            );
        }
        assert_eq!(
            leader_hw, leader_leo,
            "leader HW {leader_hw} should equal committed LEO {leader_leo}"
        );

        // Cleanup.
        let _ = admin
            .delete_shard(&ShardDeleteReq {
                shard_name: shard_name.clone(),
            })
            .await;
    }
}
