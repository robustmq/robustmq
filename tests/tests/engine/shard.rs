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
    use std::time::Duration;

    use crate::mqtt::protocol::common::create_test_env;
    use admin_server::engine::shard::{
        SegmentListReq, SegmentListResp, ShardCreateReq, ShardDeleteReq, ShardListReq, ShardListRow,
    };
    use common_base::http_response::AdminServerResponse;
    use common_base::tools::unique_id;
    use common_config::storage::StorageAdapterType;
    use metadata_struct::storage::shard::{EngineShardStatus, EngineStorageType};
    use tokio::time::sleep;

    fn check_response(result: &str, operation: &str) {
        let resp: AdminServerResponse<serde_json::Value> = serde_json::from_str(result).unwrap();
        println!(
            "{} response: code={}, error={:?}",
            operation, resp.code, resp.error
        );
        if resp.code != 0 {
            panic!(
                "{} failed: code={}, error={}",
                operation,
                resp.code,
                resp.error.unwrap_or_default()
            );
        }
    }

    #[tokio::test]
    async fn shard_test() {
        let client = create_test_env().await;
        let shard_name = unique_id();
        let config = r#"{"replica_num":1,"max_segment_size":1073741824,"retention_sec":86400,"storage_adapter_type":"Engine","engine_storage_type":"Segment"}"#.to_string();

        let create_result = client
            .create_shard(&ShardCreateReq {
                shard_name: shard_name.clone(),
                config,
            })
            .await
            .unwrap();
        check_response(&create_result, "create_shard");

        let list_req = ShardListReq {
            shard_name: Some(shard_name.clone()),
            ..Default::default()
        };
        let shard_list = client
            .get_shard_list::<_, Vec<ShardListRow>>(&list_req)
            .await
            .unwrap();
        println!("get_shard_list result: {:#?}", shard_list);
        assert_eq!(shard_list.data.len(), 1);
        let shard = &shard_list.data[0];
        assert_eq!(shard.shard_info.start_segment_seq, 0);
        assert_eq!(shard.shard_info.active_segment_seq, 0);
        assert_eq!(shard.shard_info.last_segment_seq, 0);
        assert_eq!(shard.shard_info.status, EngineShardStatus::Run);
        assert_eq!(shard.shard_info.config.replica_num, 1);
        assert_eq!(shard.shard_info.config.max_segment_size, 1073741824);
        assert_eq!(shard.shard_info.config.retention_sec, 86400);
        assert_eq!(
            shard.shard_info.config.storage_adapter_type,
            StorageAdapterType::Engine
        );
        assert_eq!(
            shard.shard_info.config.engine_storage_type,
            Some(EngineStorageType::Segment)
        );

        let segment_req = SegmentListReq {
            shard_name: shard_name.clone(),
        };
        let segment_resp_str = client.get_segment_list(&segment_req).await.unwrap();
        check_response(&segment_resp_str, "get_segment_list");
        let segment_data: AdminServerResponse<SegmentListResp> =
            serde_json::from_str(&segment_resp_str).unwrap();
        let segment_resp = segment_data.data;
        println!("get_segment_list data: {:#?}", segment_resp);
        assert_eq!(segment_resp.segment_list.len(), 1);
        assert_eq!(segment_resp.segment_list[0].segment.segment_seq, 0);

        let delete_result = client
            .delete_shard(&ShardDeleteReq {
                shard_name: shard_name.clone(),
            })
            .await
            .unwrap();
        check_response(&delete_result, "delete_shard");

        sleep(Duration::from_secs(5)).await;
        let shard_list_after_delete = client
            .get_shard_list::<_, Vec<ShardListRow>>(&list_req)
            .await
            .unwrap();
        println!(
            "get_shard_list after delete: {:#?}",
            shard_list_after_delete
        );
        assert_eq!(shard_list_after_delete.data.len(), 0);

        let segment_resp_str = client.get_segment_list(&segment_req).await.unwrap();
        check_response(&segment_resp_str, "get_segment_list_after_delete");
        let segment_data: AdminServerResponse<SegmentListResp> =
            serde_json::from_str(&segment_resp_str).unwrap();
        let segment_resp = segment_data.data;
        println!("get_segment_list after delete data: {:#?}", segment_resp);
        assert_eq!(segment_resp.segment_list.len(), 0);
    }
}
