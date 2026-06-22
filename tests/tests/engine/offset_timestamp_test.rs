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

// EngineSegment is not tested: its timestamp index relies on segment.start_timestamp
// which defaults to -1 for new segments, making precise lookup impossible with small record counts.

#[cfg(test)]
mod tests {
    use crate::engine::common::{admin_client, create_shard, engine_client, ENGINE_NODE_ID};
    use admin_server::cluster::offset::{GetOffsetByTimestampReq, GetOffsetByTimestampResp};
    use bytes::Bytes;
    use common_base::http_response::AdminServerResponse;
    use common_base::tools::now_second;
    use common_base::utils::serialize;
    use common_base::uuid::unique_id;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn write_one(
        conn: &std::sync::Arc<storage_engine::clients::manager::ClientConnectionManager>,
        shard_name: &str,
    ) -> u64 {
        let record = AdapterWriteRecord::new("", Bytes::from("msg"));
        let msg = serialize::serialize(&record).unwrap();
        let rows = conn
            .send_write(ENGINE_NODE_ID, shard_name, vec![msg])
            .await
            .expect("send_write failed");
        rows[0].offset
    }

    async fn query_offset(
        admin: &admin_server::client::AdminHttpClient,
        shard_name: &str,
        timestamp: u64,
        strategy: &str,
    ) -> u64 {
        let resp = admin
            .get_offset_by_timestamp::<_, AdminServerResponse<GetOffsetByTimestampResp>>(
                &GetOffsetByTimestampReq {
                    shard_name: shard_name.to_string(),
                    timestamp,
                    strategy: strategy.to_string(),
                },
            )
            .await
            .expect("get_offset_by_timestamp http failed");
        assert_eq!(
            resp.code, 0,
            "get_offset_by_timestamp failed: {:?}",
            resp.error
        );
        resp.data.offset
    }

    #[tokio::test]
    async fn offset_timestamp_by_memory() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineMemory"}"#;
        run_offset_timestamp_test(config).await;
    }

    #[tokio::test]
    async fn offset_timestamp_by_rocksdb() {
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_type":"EngineRocksDB"}"#;
        run_offset_timestamp_test(config).await;
    }

    // Write flow:
    //   t_a: write record → offset base   (timestamp index created at offset=0 with create_t=T_A)
    //   sleep 2s
    //   t_b: write record → offset base+1 (offset=1 ≢ 5000, no new index entry)
    //
    // Queries:
    //   T_A          → base     (record[base].create_t = T_A >= T_A, first match)
    //   T_A + 1      → base+1   (record[base].create_t = T_A < T_A+1; record[base+1].create_t = T_B > T_A+1)
    //   far_future, earliest → base   (no match, fallback to earliest)
    //   far_future, latest   → base+2 (no match, fallback to next-writable offset)
    async fn run_offset_timestamp_test(config: &str) {
        let admin = admin_client();
        let conn = engine_client();
        let shard_name = unique_id();

        create_shard(&admin, &shard_name, config).await;

        let base = write_one(&conn, &shard_name).await;
        let t_a = now_second();

        sleep(Duration::from_secs(2)).await;

        write_one(&conn, &shard_name).await;
        let t_b = now_second();

        // T_A + 1 must be strictly between T_A and T_B
        assert!(t_b > t_a, "writes too fast: t_a={} t_b={}", t_a, t_b);
        let t_mid = t_a + 1;

        // record at base has create_t = T_A → first record with create_t >= T_A
        let off = query_offset(&admin, &shard_name, t_a, "earliest").await;
        assert_eq!(off, base, "timestamp=T_A should return base offset");

        // record[base].create_t = T_A < T_mid; record[base+1].create_t = T_B > T_mid
        let off = query_offset(&admin, &shard_name, t_mid, "earliest").await;
        assert_eq!(off, base + 1, "timestamp=T_mid should return base+1");

        // no record has create_t >= far_future → fallback earliest
        let far_future = t_b + 100_000;
        let off = query_offset(&admin, &shard_name, far_future, "earliest").await;
        assert_eq!(off, base, "far_future+earliest should fallback to base");

        // fallback latest → next-writable offset = base + 2
        let off = query_offset(&admin, &shard_name, far_future, "latest").await;
        assert_eq!(off, base + 2, "far_future+latest should fallback to base+2");
    }
}
