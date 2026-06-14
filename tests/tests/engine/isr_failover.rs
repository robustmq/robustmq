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

// ISR leader-failover drill.
//
// Requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml). Marked
// `#[ignore]`; run with:
//   cargo test -p robustmq-test three_replica_leader_failover -- --ignored --nocapture
//
// Scenario: create a 3-replica shard, write to the leader, then KILL the leader
// broker process and verify the cluster fails over:
//   - segment leader switches to a surviving replica (leader_epoch bumps),
//   - the dead node leaves the ISR but is retained in the replica set,
//   - the surviving follower keeps replicating new writes from the new leader.
//
// NOTE ON TIMING: the switch is driven by the meta-service heartbeat timeout
// (`heartbeat_timeout_ms`, default 30s) — the broker does NOT actively unregister
// on graceful stop, so even `kill -INT` waits ~28-30s. The assertion window is set
// generously (60s); the test prints the actual switch latency.
//
// This test kills (and best-effort restarts) a broker process; run it standalone.

#[cfg(test)]
mod tests {
    use crate::mqtt::protocol::common::create_test_env;
    use admin_server::client::AdminHttpClient;
    use admin_server::engine::segment::{
        SegmentDetailReq, SegmentDetailResp, SegmentListReq, SegmentListResp,
    };
    use admin_server::engine::shard::{ShardCreateReq, ShardDeleteReq};
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
    use std::process::Command;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::core::cache::StorageCacheManager;
    use tokio::time::sleep;

    const NODES: [u64; 3] = [1, 2, 3];
    const REPLICA_NUM: usize = 3;
    // Graceful stop still waits for the ~30s heartbeat timeout; allow margin.
    const SWITCH_TIMEOUT: Duration = Duration::from_secs(60);

    fn engine_addr(node_id: u64) -> String {
        match node_id {
            1 => "127.0.0.1:1779",
            2 => "127.0.0.1:2779",
            3 => "127.0.0.1:3779",
            other => panic!("unknown node id {other}"),
        }
        .to_string()
    }

    fn admin_url(node_id: u64) -> String {
        match node_id {
            1 => "http://127.0.0.1:58080",
            2 => "http://127.0.0.1:58082",
            3 => "http://127.0.0.1:58083",
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
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        Arc::new(ClientConnectionManager::new(cache_manager, 2))
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
        list.iter().map(|s| s.segment.segment_seq).max().unwrap()
    }

    async fn segment_detail(
        admin: &AdminHttpClient,
        shard_name: &str,
        segment_seq: u32,
    ) -> SegmentDetailResp {
        admin
            .get_segment_detail::<_, SegmentDetailResp>(&SegmentDetailReq {
                shard_name: shard_name.to_string(),
                segment_seq,
            })
            .await
            .unwrap()
    }

    /// Write `n` records (offsets `base..base+n`) to `leader` via the engine TCP path.
    async fn write_to_leader(
        writer: &Arc<ClientConnectionManager>,
        leader: u64,
        shard_name: &str,
        base: u64,
        n: u64,
    ) {
        let mut messages = Vec::with_capacity(n as usize);
        for i in 0..n {
            let record = AdapterWriteRecord::new("", Bytes::from(format!("failover-{}", base + i)));
            messages.push(serialize::serialize(&record).unwrap());
        }
        // acks=all (-1): the write only returns once the high watermark has reached
        // these records on every ISR replica, i.e. the data is committed. This is
        // what makes a clean failover loss-free — committed data must survive.
        let mut body = WriteReqBody::new(shard_name.to_string(), messages);
        body.acks = -1;
        let write_req = WriteReq::new(body);
        let resp = writer
            .write_send(leader, StorageEnginePacket::WriteReq(write_req))
            .await
            .unwrap();
        match resp {
            StorageEnginePacket::WriteResp(r) => {
                if let Some(e) = r.header.error {
                    panic!(
                        "write to n{leader} failed: code={}, msg={}",
                        e.code, e.error
                    );
                }
            }
            other => panic!("expected WriteResp, got {other:?}"),
        }
    }

    fn replica_ids(detail: &SegmentDetailResp) -> Vec<u64> {
        detail.segment.replicas.iter().map(|r| r.node_id).collect()
    }

    #[tokio::test]
    #[ignore = "requires a running 3-node cluster; kills + restarts a broker process"]
    async fn three_replica_leader_failover() {
        let admin = create_test_env().await; // http://127.0.0.1:58080
        let shard_name = unique_id();

        // 1. Create a 3-replica RocksDB shard and wait for provisioning.
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
        sleep(Duration::from_secs(10)).await;

        // 2. Establish baseline: leader, epoch, replica set; write some data.
        let segment_seq = active_segment_seq(&admin, &shard_name).await;
        let d0 = segment_detail(&admin, &shard_name, segment_seq).await;
        assert_eq!(
            d0.segment.replicas.len(),
            REPLICA_NUM,
            "expected 3 replicas"
        );
        let old_leader = d0.segment.leader;
        let old_epoch = d0.segment.leader_epoch;
        let original_replicas = replica_ids(&d0);
        assert!(old_leader != 0, "no leader elected");
        println!(
            "baseline: leader=n{old_leader} epoch={old_epoch} isr={:?} replicas={:?}",
            d0.segment.isr, original_replicas
        );

        let writer = build_write_client();
        write_to_leader(&writer, old_leader, &shard_name, 0, 100).await;
        println!("wrote 100 records to leader n{old_leader}");

        // With acks=all the write already returned committed, but confirm every
        // replica shows LEO=HW=100 before killing: the 100 records are committed on
        // all three, so a clean failover MUST NOT lose them.
        {
            let wait = Duration::from_secs(20);
            let start = Instant::now();
            loop {
                let d = segment_detail(&admin, &shard_name, segment_seq).await;
                if d.replicas.len() == REPLICA_NUM
                    && d.replicas
                        .iter()
                        .all(|r| r.available && r.leo == 100 && r.high_watermark == 100)
                {
                    println!("all 3 replicas committed LEO=HW=100");
                    break;
                }
                if start.elapsed() > wait {
                    panic!(
                        "replicas did not commit the initial 100 within {wait:?}: {}",
                        d.replicas
                            .iter()
                            .map(|r| format!(
                                "n{}(leo={},hw={})",
                                r.node_id, r.leo, r.high_watermark
                            ))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                sleep(Duration::from_millis(300)).await;
            }
        }

        // Observe failover from a node that will survive the kill.
        let observe_node = if old_leader == 1 { 2 } else { 1 };
        let observe = AdminHttpClient::new(admin_url(observe_node));

        // 3. Gracefully kill the leader broker process.
        println!("gracefully killing leader node{old_leader} (pkill -INT)");
        let status = Command::new("pkill")
            .args(["-INT", "-f", &format!("server-{old_leader}.toml")])
            .status()
            .expect("failed to invoke pkill");
        assert!(status.success(), "pkill did not signal the leader process");
        let t0 = Instant::now();

        // 4. Poll until the segment leader switches.
        let (switch_latency, d) = loop {
            let d = segment_detail(&observe, &shard_name, segment_seq).await;
            let l = d.segment.leader;
            if l != old_leader && l != 0 {
                break (t0.elapsed(), d);
            }
            if t0.elapsed() > SWITCH_TIMEOUT {
                panic!("no leader switch within {SWITCH_TIMEOUT:?} (still leader=n{old_leader})");
            }
            sleep(Duration::from_secs(1)).await;
        };
        let new_leader = d.segment.leader;
        println!(
            "LEADER SWITCHED n{old_leader} -> n{new_leader} after {switch_latency:?} \
             (epoch {old_epoch} -> {}, isr={:?}, replicas={:?})",
            d.segment.leader_epoch,
            d.segment.isr,
            replica_ids(&d)
        );

        // 5. Failover assertions.
        assert_ne!(new_leader, old_leader);
        assert!(
            original_replicas.contains(&new_leader),
            "new leader n{new_leader} must be one of the original replicas {original_replicas:?}"
        );
        assert!(
            d.segment.leader_epoch > old_epoch,
            "leader_epoch must advance on failover ({old_epoch} -> {})",
            d.segment.leader_epoch
        );
        assert!(
            !d.segment.isr.contains(&old_leader),
            "dead leader n{old_leader} must be removed from ISR (isr={:?})",
            d.segment.isr
        );
        assert!(
            replica_ids(&d).contains(&old_leader),
            "dead node n{old_leader} should be retained in the replica set (temporary offline)"
        );
        // No data loss across the switch: poll until the surviving replicas settle
        // (a freshly promoted leader briefly resets ISR to itself and a follower may
        // re-sync), then require every survivor to still hold all 100 COMMITTED
        // records — leo=hw=100, log start unchanged at 0.
        {
            let settle = Duration::from_secs(20);
            let s0 = Instant::now();
            loop {
                let dd = segment_detail(&observe, &shard_name, segment_seq).await;
                let survivors: Vec<_> = dd
                    .replicas
                    .iter()
                    .filter(|r| r.node_id != old_leader)
                    .collect();
                let ok = survivors.len() == REPLICA_NUM - 1
                    && survivors.iter().all(|r| {
                        r.available
                            && r.leo == 100
                            && r.high_watermark == 100
                            && r.log_start_offset == 0
                    });
                if ok {
                    println!("no data loss: both survivors hold committed LEO=HW=100");
                    break;
                }
                if s0.elapsed() > settle {
                    panic!(
                        "committed data lost/inconsistent on survivors after switch: {}",
                        survivors
                            .iter()
                            .map(|r| format!(
                                "n{}(leo={},hw={},lso={})",
                                r.node_id, r.leo, r.high_watermark, r.log_start_offset
                            ))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                sleep(Duration::from_millis(500)).await;
            }
        }

        // 6. Survivors keep working: write more to the NEW leader and verify the
        //    surviving follower replicates up to the new leader's LEO.
        write_to_leader(&writer, new_leader, &shard_name, 100, 100).await;
        let expected_leo = 200u64;
        println!(
            "wrote 100 more records to new leader n{new_leader} (expected LEO {expected_leo})"
        );

        let deadline = Duration::from_secs(30);
        let start = Instant::now();
        loop {
            let d = segment_detail(&observe, &shard_name, segment_seq).await;
            // Surviving replicas = everyone except the killed old leader.
            let survivors: Vec<_> = d
                .replicas
                .iter()
                .filter(|r| r.node_id != old_leader)
                .collect();
            let leader_leo = survivors
                .iter()
                .find(|r| r.node_id == new_leader)
                .map(|r| r.leo)
                .unwrap_or(0);
            let all_caught = survivors.len() == REPLICA_NUM - 1
                && survivors.iter().all(|r| {
                    r.available
                        && r.leo == leader_leo
                        && r.high_watermark == leader_leo
                        && r.log_start_offset == 0
                });

            println!(
                "[{:?}] new_leader_leo={leader_leo} | {}",
                start.elapsed(),
                d.replicas
                    .iter()
                    .map(|r| format!(
                        "n{}(leo={},hw={},isr={},avail={})",
                        r.node_id, r.leo, r.high_watermark, r.in_isr, r.available
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            if all_caught && leader_leo >= expected_leo {
                println!("survivors converged: 2 live replicas at LEO=HW={leader_leo}");
                break;
            }
            if start.elapsed() > deadline {
                panic!(
                    "survivors did not catch up within {deadline:?}: replicas={:#?}",
                    d.replicas
                );
            }
            sleep(Duration::from_millis(500)).await;
        }

        // 7. Restart the killed node and verify it REJOINS the ISR. A restarted
        //    follower that missed the leader-switch notification must resume
        //    replication (reconcile self-heal) and re-enter the ISR.
        println!("restarting node{old_leader}");
        // cargo test runs with CWD = the test package dir, not the workspace root,
        // so resolve the binary/config/data paths against the repo root and run the
        // broker there (it writes ./data relative to its CWD).
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root");
        let bin = repo_root.join("target/debug/broker-server");
        let log = std::fs::File::create(format!("/tmp/n{old_leader}.log")).unwrap();
        let _ = Command::new(&bin)
            .current_dir(repo_root)
            .args([
                "--conf",
                &format!("config/cluster/server-{old_leader}.toml"),
            ])
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn(); // dropped Child keeps running (not waited on)

        // Rejoin is driven by the metadata reconcile thread (~30s interval), so
        // allow a generous window.
        let rejoin_deadline = Duration::from_secs(75);
        let start = Instant::now();
        loop {
            let d = segment_detail(&observe, &shard_name, segment_seq).await;
            if d.segment.isr.contains(&old_leader) {
                println!(
                    "node{old_leader} REJOINED ISR after {:?}: isr={:?}",
                    start.elapsed(),
                    d.segment.isr
                );
                break;
            }
            if start.elapsed() > rejoin_deadline {
                panic!(
                    "node{old_leader} did not rejoin ISR within {rejoin_deadline:?} (isr={:?})",
                    d.segment.isr
                );
            }
            sleep(Duration::from_secs(2)).await;
        }

        // Rejoined node must have fully caught up — same data as the survivors, no loss.
        let dr = segment_detail(&observe, &shard_name, segment_seq).await;
        let rep = dr
            .replicas
            .iter()
            .find(|r| r.node_id == old_leader)
            .expect("rejoined replica present");
        assert_eq!(
            rep.leo, expected_leo,
            "rejoined n{old_leader} LEO must be {expected_leo}"
        );
        assert_eq!(
            rep.high_watermark, expected_leo,
            "rejoined n{old_leader} HW must be {expected_leo}"
        );
        assert_eq!(
            rep.log_start_offset, 0,
            "rejoined n{old_leader} log_start_offset must be 0"
        );

        let _ = admin
            .delete_shard(&ShardDeleteReq {
                shard_name: shard_name.clone(),
            })
            .await;
    }
}
