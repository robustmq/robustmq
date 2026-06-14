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

// ISR rolling-restart chaos drill.
//
// Requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml). Marked
// `#[ignore]`; run with:
//   cargo test -p robustmq-test three_replica_chaos_rolling_kill -- --ignored --nocapture
//
// Simulates a rolling restart of the whole cluster: each round restarts ONE node
// (rotating 1 -> 2 -> 3 -> 1 ...), so the meta-service Raft always keeps a 2/3
// quorum. Restarting the current leader forces a leader switch, so over 10 rounds
// leadership moves around frequently. With acks=all writes (committed) both before
// and during each 1-node outage, the test verifies the ISR machinery stays correct
// under this churn.
//
// The review conditions are the point — each is chosen to catch a real defect:
//   - hw <= leo on EVERY observation               → HW/offset bookkeeping corruption
//   - all replicas leo == hw == cumulative         → data loss / divergence / lag
//   - lso == 0                                      → unexpected truncation/retention
//   - replicas == {1,2,3} always                    → replica drop/migration
//   - ISR drops only the dead node, then recovers   → wrong ISR shrink / failure to rejoin
//   - leader_epoch is monotonic, bumps on switch    → epoch not advancing (fencing broken)
//   - all 3 nodes agree on leader/epoch/ISR         → split-brain / stale metadata view
//   - acks=all commits in both full & degraded ISR  → writes stalling (availability)
//   - final read-back returns `cumulative` records  → counter-correct-but-content-lost
//   - node logs contain no blacklisted lines        → raft panics, watchdog kills, etc.
//
// This test kills + restarts broker processes; run it standalone.

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
    use protocol::storage::protocol::{
        ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType, WriteReq,
        WriteReqBody,
    };
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::core::cache::StorageCacheManager;
    use tokio::time::sleep;

    const NODES: [u64; 3] = [1, 2, 3];
    const REPLICA_NUM: usize = 3;
    const ROUNDS: u64 = 10;
    const BATCH: u64 = 30;

    // Log lines that must NEVER appear — each is an unambiguous defect. Benign
    // rolling-restart noise (Unreachable node, fetcher retry to a just-killed
    // leader, segment-detail fan-out to a dead node, reconcile self-heal warn,
    // "Termination signal received", diverged-tail truncation) is intentionally
    // NOT listed.
    const LOG_BLACKLIST: &[&str] = &[
        "invalid state: expect", // raft purge_upto > snapshot regression
        "last_log_id=None",      // raft log read back empty on restart
        "Clean the hole",        // raft storage hole
        "forcing exit",          // shutdown watchdog fired (ungraceful)
        "acks=all timed out",    // committed write never acked
        "NotEnoughReplicas",     // acks=all rejected
    ];

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

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf()
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

    fn node_running(node_id: u64) -> bool {
        Command::new("pgrep")
            .args(["-f", &format!("server-{node_id}.toml")])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false)
    }

    async fn kill_node(node_id: u64) {
        // Graceful stop only (kill -INT); kill -9 leaves a stuck grpc socket on
        // this host that blocks the port on restart.
        let _ = Command::new("pkill")
            .args(["-INT", "-f", &format!("server-{node_id}.toml")])
            .status();
        let t0 = Instant::now();
        while node_running(node_id) {
            if t0.elapsed() > Duration::from_secs(25) {
                panic!("node{node_id} did not exit on kill -INT within 25s");
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    fn restart_node(node_id: u64) {
        let root = repo_root();
        let bin = root.join("target/debug/broker-server");
        // Append so the per-node log accumulates the whole run for log analysis.
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("/tmp/n{node_id}.log"))
            .unwrap();
        let _ = Command::new(&bin)
            .current_dir(&root)
            .args(["--conf", &format!("config/cluster/server-{node_id}.toml")])
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn(); // dropped Child keeps running (not waited on)
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

    /// segment_detail + the universal `hw <= leo` invariant enforced on every read.
    async fn detail(
        admin: &AdminHttpClient,
        shard_name: &str,
        segment_seq: u32,
    ) -> SegmentDetailResp {
        let d = admin
            .get_segment_detail::<_, SegmentDetailResp>(&SegmentDetailReq {
                shard_name: shard_name.to_string(),
                segment_seq,
            })
            .await
            .unwrap();
        for r in &d.replicas {
            assert!(
                r.high_watermark <= r.leo,
                "INVARIANT hw<=leo violated on n{}: hw={} leo={} (shard {shard_name})",
                r.node_id,
                r.high_watermark,
                r.leo
            );
        }
        d
    }

    async fn write_acks_all(
        writer: &Arc<ClientConnectionManager>,
        leader: u64,
        shard_name: &str,
        label_base: u64,
        n: u64,
    ) {
        let mut messages = Vec::with_capacity(n as usize);
        for i in 0..n {
            let record =
                AdapterWriteRecord::new("", Bytes::from(format!("chaos-{}", label_base + i)));
            messages.push(serialize::serialize(&record).unwrap());
        }
        let mut body = WriteReqBody::new(shard_name.to_string(), messages);
        body.acks = -1;
        let resp = writer
            .write_send(leader, StorageEnginePacket::WriteReq(WriteReq::new(body)))
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

    /// Read records from `offset` 0 up, return how many are actually readable.
    async fn read_back_count(
        writer: &Arc<ClientConnectionManager>,
        leader: u64,
        shard_name: &str,
        want: u64,
    ) -> u64 {
        let mut got = 0u64;
        loop {
            let req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
                shard_name.to_string(),
                ReadType::Offset,
                false,
                ReadReqFilter::by_offset(got),
                ReadReqOptions::new(64 * 1024 * 1024, want),
            )]));
            let resp = writer
                .read_send(leader, StorageEnginePacket::ReadReq(req))
                .await
                .unwrap();
            let n = match resp {
                StorageEnginePacket::ReadResp(r) => {
                    if let Some(e) = r.header.error {
                        panic!("read failed: code={}, msg={}", e.code, e.error);
                    }
                    r.body.messages.len() as u64
                }
                other => panic!("expected ReadResp, got {other:?}"),
            };
            if n == 0 {
                break;
            }
            got += n;
            if got >= want {
                break;
            }
        }
        got
    }

    fn replica_ids(d: &SegmentDetailResp) -> Vec<u64> {
        let mut v: Vec<u64> = d.segment.replicas.iter().map(|r| r.node_id).collect();
        v.sort_unstable();
        v
    }

    fn sorted_isr(d: &SegmentDetailResp) -> Vec<u64> {
        let mut v = d.segment.isr.clone();
        v.sort_unstable();
        v
    }

    fn replicas_line(d: &SegmentDetailResp) -> String {
        d.replicas
            .iter()
            .map(|r| {
                format!(
                    "n{}(leo={},hw={},lso={},isr={},avail={})",
                    r.node_id, r.leo, r.high_watermark, r.log_start_offset, r.in_isr, r.available
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Scan all node logs for blacklisted lines; returns the violations found.
    fn scan_logs() -> Vec<String> {
        let mut hits = Vec::new();
        for id in NODES {
            let path = format!("/tmp/n{id}.log");
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in content.lines() {
                if LOG_BLACKLIST.iter().any(|p| line.contains(p)) {
                    // strip ANSI for readability
                    let clean: String = line.replace('\u{1b}', "").replace("[0m", "");
                    hits.push(format!("n{id}: {}", clean.trim()));
                }
            }
        }
        hits
    }

    /// Poll until all 3 replicas are in ISR, available, and leo==hw==target, lso==0.
    async fn wait_full_and_caught_up(
        obs: &AdminHttpClient,
        shard_name: &str,
        segment_seq: u32,
        target: u64,
        stage: &str,
    ) -> SegmentDetailResp {
        let deadline = Duration::from_secs(90);
        let start = Instant::now();
        loop {
            let d = detail(obs, shard_name, segment_seq).await;
            let full =
                d.segment.isr.len() == REPLICA_NUM && d.segment.replicas.len() == REPLICA_NUM;
            let caught = d.replicas.len() == REPLICA_NUM
                && d.replicas.iter().all(|r| {
                    r.available
                        && r.in_isr
                        && r.leo == target
                        && r.high_watermark == target
                        && r.log_start_offset == 0
                });
            if full && caught {
                return d;
            }
            if start.elapsed() > deadline {
                panic!(
                    "[{stage}] not full-ISR + leo=hw={target} within {deadline:?}: leader=n{} isr={:?} | {}",
                    d.segment.leader,
                    d.segment.isr,
                    replicas_line(&d)
                );
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    #[tokio::test]
    #[ignore = "requires a running 3-node cluster; repeatedly restarts broker processes"]
    async fn three_replica_chaos_rolling_kill() {
        let admin = create_test_env().await; // http://127.0.0.1:58080
        assert!(
            repo_root().join("target/debug/broker-server").exists(),
            "broker-server binary not found; build it first"
        );

        let shard_name = unique_id();
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

        let segment_seq = active_segment_seq(&admin, &shard_name).await;
        let writer = build_write_client();
        let mut cumulative: u64 = 0;
        let mut last_epoch: u32 = 0;
        let mut switch_count: u32 = 0;

        for round in 1..=ROUNDS {
            let victim = ((round - 1) % 3) + 1;
            let observe_node = if victim == 1 { 2 } else { 1 };
            let obs = AdminHttpClient::new(admin_url(observe_node));

            // Precondition: cluster fully healthy and caught up.
            let d0 =
                wait_full_and_caught_up(&obs, &shard_name, segment_seq, cumulative, "round-start")
                    .await;
            let leader = d0.segment.leader;

            // (1) all 3 up — committed write to the current leader.
            write_acks_all(&writer, leader, &shard_name, cumulative, BATCH).await;
            cumulative += BATCH;
            wait_full_and_caught_up(&obs, &shard_name, segment_seq, cumulative, "after-write-A")
                .await;

            let was_leader = victim == leader;
            println!(
                "===== round {round}: leader=n{leader}, restarting n{victim} (was_leader={was_leader}), cumulative={cumulative} ====="
            );

            // (2) kill one node (rolling restart step 1: stop).
            kill_node(victim).await;

            // (3) cluster reacts: dead node leaves ISR and (if it was leader) leadership
            //     moves to a survivor — heartbeat-driven (~30s).
            {
                let deadline = Duration::from_secs(55);
                let t0 = Instant::now();
                loop {
                    let d = detail(&obs, &shard_name, segment_seq).await;
                    let leader_ok = d.segment.leader != victim && d.segment.leader != 0;
                    if !d.segment.isr.contains(&victim) && leader_ok {
                        if was_leader {
                            assert!(
                                d.segment.leader_epoch > last_epoch,
                                "leader switched but epoch did not advance ({last_epoch} -> {})",
                                d.segment.leader_epoch
                            );
                        }
                        println!(
                            "  n{victim} left ISR after {:?}: leader=n{} epoch={} isr={:?}",
                            t0.elapsed(),
                            d.segment.leader,
                            d.segment.leader_epoch,
                            d.segment.isr
                        );
                        break;
                    }
                    if t0.elapsed() > deadline {
                        panic!(
                            "n{victim} not removed from ISR / leader not switched within {deadline:?}: leader=n{} isr={:?}",
                            d.segment.leader, d.segment.isr
                        );
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            }

            // No loss while degraded: the two survivors still hold all committed data.
            {
                let d = detail(&obs, &shard_name, segment_seq).await;
                assert_eq!(
                    replica_ids(&d),
                    vec![1, 2, 3],
                    "replica set must stay intact while degraded"
                );
                for r in d.replicas.iter().filter(|r| r.node_id != victim) {
                    assert!(r.available, "survivor n{} unavailable", r.node_id);
                    assert_eq!(
                        r.leo, cumulative,
                        "survivor n{} leo != {cumulative}",
                        r.node_id
                    );
                    assert_eq!(
                        r.high_watermark, cumulative,
                        "survivor n{} hw != {cumulative}",
                        r.node_id
                    );
                    assert_eq!(r.log_start_offset, 0, "survivor n{} lso != 0", r.node_id);
                }
            }

            // (4) degraded write: with one node down, acks=all must still commit on the
            //     two surviving ISR members (catches writes stalling when ISR can't shrink).
            let leader2 = detail(&obs, &shard_name, segment_seq).await.segment.leader;
            write_acks_all(&writer, leader2, &shard_name, cumulative, BATCH).await;
            cumulative += BATCH;
            {
                let deadline = Duration::from_secs(20);
                let t0 = Instant::now();
                loop {
                    let d = detail(&obs, &shard_name, segment_seq).await;
                    let survivors: Vec<_> =
                        d.replicas.iter().filter(|r| r.node_id != victim).collect();
                    if survivors.len() == REPLICA_NUM - 1
                        && survivors.iter().all(|r| {
                            r.available && r.leo == cumulative && r.high_watermark == cumulative
                        })
                    {
                        break;
                    }
                    if t0.elapsed() > deadline {
                        panic!("degraded acks=all write did not commit on survivors within {deadline:?}: {}", replicas_line(&d));
                    }
                    sleep(Duration::from_millis(500)).await;
                }
                println!("  degraded write committed on 2 survivors at leo=hw={cumulative}");
            }

            // (5) restart the node (rolling restart step 2); (6) it must rejoin and catch
            //     up everything it missed while down.
            restart_node(victim);
            let d =
                wait_full_and_caught_up(&obs, &shard_name, segment_seq, cumulative, "after-rejoin")
                    .await;

            // (7) review conditions.
            assert_eq!(
                replica_ids(&d),
                vec![1, 2, 3],
                "replica set must be {{1,2,3}}"
            );
            assert_eq!(
                sorted_isr(&d),
                vec![1, 2, 3],
                "ISR must recover to {{1,2,3}}"
            );
            for r in &d.replicas {
                assert!(r.in_isr, "n{} not in ISR", r.node_id);
                assert!(r.available, "n{} not available", r.node_id);
                assert_eq!(r.leo, cumulative, "n{} leo != {cumulative}", r.node_id);
                assert_eq!(
                    r.high_watermark, cumulative,
                    "n{} hw != {cumulative}",
                    r.node_id
                );
                assert_eq!(r.log_start_offset, 0, "n{} lso != 0", r.node_id);
            }

            // leader_epoch monotonic across the whole run; count switches.
            assert!(
                d.segment.leader_epoch >= last_epoch,
                "leader_epoch went backward: {last_epoch} -> {}",
                d.segment.leader_epoch
            );
            if d.segment.leader_epoch > last_epoch {
                switch_count += 1;
            }
            last_epoch = d.segment.leader_epoch;

            // Cross-node agreement: every live node must report the same leader / epoch / ISR.
            {
                let mut views: Vec<(u64, u64, u32, Vec<u64>)> = Vec::new();
                for n in NODES {
                    let dn = detail(
                        &AdminHttpClient::new(admin_url(n)),
                        &shard_name,
                        segment_seq,
                    )
                    .await;
                    views.push((
                        n,
                        dn.segment.leader,
                        dn.segment.leader_epoch,
                        sorted_isr(&dn),
                    ));
                }
                let (_, l0, e0, isr0) = views[0].clone();
                for (n, l, e, isr) in &views {
                    assert_eq!(
                        *l, l0,
                        "node{n} sees leader=n{l}, node{} sees n{l0}",
                        views[0].0
                    );
                    assert_eq!(
                        *e, e0,
                        "node{n} sees epoch={e}, node{} sees {e0}",
                        views[0].0
                    );
                    assert_eq!(
                        *isr, isr0,
                        "node{n} sees isr={isr:?}, node{} sees {isr0:?}",
                        views[0].0
                    );
                }
            }

            // No blacklisted log lines may appear at any point.
            let hits = scan_logs();
            assert!(
                hits.is_empty(),
                "unexpected log lines after round {round}:\n{}",
                hits.join("\n")
            );

            println!(
                "===== round {round} OK: leader=n{} epoch={} isr={:?} all leo=hw={cumulative} lso=0 =====",
                d.segment.leader, d.segment.leader_epoch, d.segment.isr
            );
        }

        // Final content read-back: every committed record must be readable.
        // All three nodes are up at the end; query via node 1.
        let final_admin = AdminHttpClient::new(admin_url(1));
        let leader = detail(&final_admin, &shard_name, segment_seq)
            .await
            .segment
            .leader;
        let read = read_back_count(&writer, leader, &shard_name, cumulative).await;
        assert_eq!(
            read, cumulative,
            "read-back returned {read} records, expected {cumulative}"
        );

        let hits = scan_logs();
        assert!(
            hits.is_empty(),
            "unexpected log lines at end:\n{}",
            hits.join("\n")
        );

        println!(
            "CHAOS COMPLETE: {ROUNDS} rounds, {cumulative} committed records read back OK, {switch_count} leader-epoch switches, all replicas consistent"
        );
        let _ = admin
            .delete_shard(&ShardDeleteReq {
                shard_name: shard_name.clone(),
            })
            .await;
    }
}
