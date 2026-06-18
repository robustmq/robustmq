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

// Meta-service Raft cluster failover drill.
//
// Requires a running 3-node cluster (config/cluster/server-{1,2,3}.toml) whose nodes
// log to /tmp/n{1,2,3}.log. Marked `#[ignore]`; run with:
//   cargo test -p robustmq-test raft_cluster_drill -- --ignored --nocapture
//
// Each round: write a tenant through Raft → kill the current metadata leader (graceful
// SIGINT) → assert re-election among survivors → restart it → assert full recovery and
// three-node consensus convergence, then verify every tenant written so far is readable
// on all three nodes (state-machine durability) and last_applied is monotonic.
//
// Review conditions:
//   - membership stays {1,2,3} on every observation   → stop must not scale-in
//   - killing the leader triggers re-election          → quorum re-elects
//   - restarted node rejoins and all nodes converge    → recover, no manual re-join
//   - last_applied monotonic per round                 → durable apply progress
//   - tenants written before a restart read back       → state machine survives restart
//   - node exits within the graceful window on SIGINT  → graceful shutdown
//   - node logs contain no blacklisted lines           → raft panics / storage holes

#[cfg(test)]
mod tests {
    use common_base::uuid::unique_id;
    use reqwest::Client;
    use serde_json::{json, Value};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    const NODES: [u64; 3] = [1, 2, 3];
    const METADATA_GROUP: &str = "metadata_0";

    // Lines that must NEVER appear in a node log — each is an unambiguous defect
    // (raft storage regression, empty log read-back, or an ungraceful watchdog exit).
    const LOG_BLACKLIST: &[&str] = &[
        "invalid state: expect",
        "last_log_id=None",
        "Clean the hole",
        "forcing exit",
        "panicked at",
    ];

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

    fn node_log(node_id: u64) -> String {
        format!("/tmp/n{node_id}.log")
    }

    fn node_running(node_id: u64) -> bool {
        Command::new("pgrep")
            .args(["-f", &format!("server-{node_id}.toml")])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false)
    }

    // Graceful stop only — SIGINT, then wait for the process to exit on its own.
    async fn kill_node(node_id: u64) {
        let _ = Command::new("pkill")
            .args(["-INT", "-f", &format!("server-{node_id}.toml")])
            .status();
        let t0 = Instant::now();
        while node_running(node_id) {
            if t0.elapsed() > Duration::from_secs(30) {
                panic!("node{node_id} did not exit on SIGINT within 30s (not graceful)");
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    // Always appends to /tmp/nX.log so scan_logs covers the whole multi-restart run.
    fn restart_node(node_id: u64) {
        let root = repo_root();
        let bin = root.join("target/debug/broker-server");
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(node_log(node_id))
            .unwrap();
        let _ = Command::new(&bin)
            .current_dir(&root)
            .args(["--conf", &format!("config/cluster/server-{node_id}.toml")])
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn();
    }

    // GET /api/info -> data.meta map (raft group -> state), or None if unreachable.
    async fn meta_info(client: &Client, node_id: u64) -> Option<Value> {
        let url = format!("{}/api/info", admin_url(node_id));
        let resp = client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .ok()?;
        let v: Value = resp.json().await.ok()?;
        v.get("data")?.get("meta").cloned()
    }

    // Current metadata_0 leader as reported by any reachable node (0/None = no leader).
    async fn meta_leader(client: &Client) -> Option<u64> {
        for n in NODES {
            if let Some(meta) = meta_info(client, n).await {
                let leader = meta
                    .get(METADATA_GROUP)
                    .and_then(|g| g.get("current_leader"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                if leader != 0 {
                    return Some(leader);
                }
            }
        }
        None
    }

    // Flatten membership_config.membership.configs ([[1,2,3]]) to a sorted id list.
    fn members_of(group: &Value) -> Vec<u64> {
        let mut ids = Vec::new();
        if let Some(configs) = group
            .get("membership_config")
            .and_then(|m| m.get("membership"))
            .and_then(|m| m.get("configs"))
            .and_then(|c| c.as_array())
        {
            for cfg in configs {
                if let Some(arr) = cfg.as_array() {
                    for id in arr {
                        if let Some(u) = id.as_u64() {
                            ids.push(u);
                        }
                    }
                }
            }
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    async fn create_tenant(client: &Client, node_id: u64, name: &str) -> bool {
        let url = format!("{}/api/cluster/tenant/create", admin_url(node_id));
        match client
            .post(&url)
            .json(&json!({ "tenant_name": name, "desc": "raft-drill" }))
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r
                .json::<Value>()
                .await
                .ok()
                .and_then(|v| v.get("code").and_then(|c| c.as_i64()))
                .map(|c| c == 0)
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    async fn list_tenants(client: &Client, node_id: u64) -> Vec<String> {
        // list endpoints default to limit=10; pass a large limit so accumulated tenants
        // are not paginated out and falsely reported missing.
        let url = format!(
            "{}/api/cluster/tenant/list?limit=100000",
            admin_url(node_id)
        );
        let mut out = Vec::new();
        if let Ok(r) = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            if let Ok(v) = r.json::<Value>().await {
                if let Some(arr) = v
                    .get("data")
                    .and_then(|d| d.get("data"))
                    .and_then(|a| a.as_array())
                {
                    for t in arr {
                        if let Some(n) = t.get("tenant_name").and_then(|x| x.as_str()) {
                            out.push(n.to_string());
                        }
                    }
                }
            }
        }
        out
    }

    fn scan_logs() -> Vec<String> {
        let mut hits = Vec::new();
        for id in NODES {
            let Ok(content) = std::fs::read_to_string(node_log(id)) else {
                continue;
            };
            for line in content.lines() {
                if LOG_BLACKLIST.iter().any(|p| line.contains(p)) {
                    let clean: String = line.replace('\u{1b}', "").replace("[0m", "");
                    hits.push(format!("n{id}: {}", clean.trim()));
                }
            }
        }
        hits
    }

    // Poll until all three nodes report the same metadata_0 leader / term / last_applied
    // and membership {1,2,3}. Returns (leader, term, last_applied).
    async fn wait_converged(client: &Client, stage: &str) -> (u64, u64, u64) {
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            let mut leaders = Vec::new();
            let mut terms = Vec::new();
            let mut applieds = Vec::new();
            let mut members_ok = true;
            let mut all_reachable = true;

            for n in NODES {
                match meta_info(client, n)
                    .await
                    .and_then(|m| m.get(METADATA_GROUP).cloned())
                {
                    Some(g) => {
                        leaders.push(
                            g.get("current_leader")
                                .and_then(|x| x.as_u64())
                                .unwrap_or(0),
                        );
                        terms.push(g.get("current_term").and_then(|x| x.as_u64()).unwrap_or(0));
                        applieds.push(
                            g.get("last_applied")
                                .and_then(|x| x.get("index"))
                                .and_then(|x| x.as_u64())
                                .unwrap_or(0),
                        );
                        if members_of(&g) != vec![1, 2, 3] {
                            members_ok = false;
                        }
                    }
                    None => all_reachable = false,
                }
            }

            let converged = all_reachable
                && members_ok
                && leaders.len() == 3
                && leaders.iter().all(|&l| l != 0 && l == leaders[0])
                && terms.iter().all(|&t| t == terms[0])
                && applieds.iter().all(|&a| a == applieds[0]);

            if converged {
                return (leaders[0], terms[0], applieds[0]);
            }
            if Instant::now() >= deadline {
                panic!(
                    "[{stage}] cluster did not converge within 90s \
                     (leaders={leaders:?}, terms={terms:?}, last_applied={applieds:?}, members_ok={members_ok})"
                );
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    #[tokio::test]
    #[ignore = "requires a running 3-node cluster; repeatedly restarts broker processes"]
    async fn raft_cluster_drill() {
        const ROUNDS: u64 = 5;
        let client = Client::new();

        // ── preflight: cluster converged on membership {1,2,3} with a leader ──
        let (leader0, term0, applied0) = wait_converged(&client, "preflight").await;
        println!(
            "preflight OK: leader=n{leader0} term={term0} last_applied={applied0} members=[1,2,3]"
        );

        let mut expect_tenants: Vec<String> = Vec::new();
        let mut last_applied_floor = 0u64;

        for round in 1..=ROUNDS {
            println!("===== raft drill round {round} =====");

            // ── (1) write a tenant through Raft (auto-forwarded to the leader) ──
            let tenant = format!("raftdrill-{}", unique_id());
            assert!(
                create_tenant(&client, 1, &tenant).await,
                "round {round}: tenant '{tenant}' create failed"
            );
            expect_tenants.push(tenant.clone());

            // ── (2) kill the current metadata leader (graceful) ──
            let leader = meta_leader(&client)
                .await
                .expect("no metadata leader before kill");
            println!("  round {round}: killing leader n{leader}");
            kill_node(leader).await;

            // ── (3) survivors must re-elect a new leader; membership unchanged ──
            {
                let t0 = Instant::now();
                loop {
                    if let Some(nl) = meta_leader(&client).await {
                        if nl != leader && nl != 0 {
                            // confirm survivors still see membership {1,2,3}
                            let survivor = if leader == 1 { 2 } else { 1 };
                            let members = meta_info(&client, survivor)
                                .await
                                .and_then(|m| m.get(METADATA_GROUP).cloned())
                                .map(|g| members_of(&g))
                                .unwrap_or_default();
                            assert_eq!(
                                members,
                                vec![1, 2, 3],
                                "round {round}: membership changed after killing leader n{leader}"
                            );
                            println!("  round {round}: re-elected leader n{nl} (was n{leader})");
                            break;
                        }
                    }
                    if t0.elapsed() > Duration::from_secs(45) {
                        panic!("round {round}: no re-election within 45s after killing leader n{leader}");
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            }

            // ── (4) restart the killed node; it recovers without a manual re-join ──
            restart_node(leader);

            // ── (5) all three nodes converge (leader/term/last_applied agree, members {1,2,3}) ──
            let (l, t, applied) = wait_converged(&client, &format!("round-{round}-recover")).await;

            // ── (6) last_applied is monotonic across rounds ──
            assert!(
                applied >= last_applied_floor,
                "round {round}: last_applied went backward ({last_applied_floor} -> {applied})"
            );
            last_applied_floor = applied;

            // ── (7) every tenant written so far is readable on all three nodes ──
            for n in NODES {
                let have = list_tenants(&client, n).await;
                for tn in &expect_tenants {
                    assert!(
                        have.iter().any(|x| x == tn),
                        "round {round}: tenant '{tn}' missing on node{n} after restart"
                    );
                }
            }

            // ── (8) no blacklisted log lines ──
            let hits = scan_logs();
            assert!(
                hits.is_empty(),
                "round {round}: blacklisted log lines:\n{}",
                hits.join("\n")
            );

            println!(
                "===== round {round} OK: leader=n{l} term={t} last_applied={applied} \
                 members=[1,2,3], {} tenants durable ====",
                expect_tenants.len()
            );
        }

        println!(
            "RAFT DRILL COMPLETE: {ROUNDS} rounds, membership stable [1,2,3], \
             consensus converged each round, {} tenants durable across leader kills/restarts",
            expect_tenants.len()
        );
    }
}
