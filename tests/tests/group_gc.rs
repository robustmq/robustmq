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

#[cfg(test)]
mod tests {
    use admin_server::client::AdminHttpClient;
    use admin_server::cluster::offset::{
        CommitOffsetReq, GetOffsetByGroupReq, GetOffsetByGroupResp,
    };
    use common_base::uuid::unique_id;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn create_test_env() -> AdminHttpClient {
        AdminHttpClient::new("http://127.0.0.1:8080")
    }

    #[tokio::test]
    async fn group_test() {
        let client = create_test_env().await;
        let tenant = "public".to_string();
        let group_name = format!("test-group-{}", unique_id());
        let shard_name = format!("shard-{}", unique_id());

        // ── 1. get group — should be empty ────────────────────────────────────
        let get_req = GetOffsetByGroupReq {
            tenant: tenant.clone(),
            group_name: group_name.clone(),
        };
        let resp = client
            .get_offset_by_group::<_, GetOffsetByGroupResp>(&get_req)
            .await
            .unwrap();
        println!("get group before commit: {:#?}", resp);
        assert!(
            resp.offsets.is_empty(),
            "group should not exist before commit"
        );

        // ── 2. commit offset (shard → 10) ─────────────────────────────────────
        let mut offsets = HashMap::new();
        offsets.insert(shard_name.clone(), 10u64);
        let commit_req = CommitOffsetReq {
            tenant: tenant.clone(),
            group_name: group_name.clone(),
            offsets,
        };
        client.commit_offset(&commit_req).await.unwrap();
        sleep(Duration::from_secs(1)).await;

        // ── 3. get group — should have offset=10 ──────────────────────────────
        let resp = client
            .get_offset_by_group::<_, GetOffsetByGroupResp>(&get_req)
            .await
            .unwrap();
        println!("get group after first commit: {:#?}", resp);
        assert_eq!(resp.offsets.len(), 1, "expected 1 offset entry");
        let entry = &resp.offsets[0];
        assert_eq!(entry.shard_name, shard_name);
        assert_eq!(entry.offset, 10);

        // ── 4. commit offset (shard → 20) ─────────────────────────────────────
        let mut offsets2 = HashMap::new();
        offsets2.insert(shard_name.clone(), 20u64);
        let commit_req2 = CommitOffsetReq {
            tenant: tenant.clone(),
            group_name: group_name.clone(),
            offsets: offsets2,
        };
        client.commit_offset(&commit_req2).await.unwrap();
        sleep(Duration::from_secs(1)).await;

        // ── 5. get group — offset should be updated to 20 ─────────────────────
        let resp = client
            .get_offset_by_group::<_, GetOffsetByGroupResp>(&get_req)
            .await
            .unwrap();
        println!("get group after second commit: {:#?}", resp);
        assert_eq!(resp.offsets.len(), 1, "expected 1 offset entry");
        let entry = &resp.offsets[0];
        assert_eq!(entry.shard_name, shard_name);
        assert_eq!(entry.offset, 20, "offset should be updated to 20");
    }
}
