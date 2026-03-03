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

use async_trait::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;

#[async_trait]
pub trait ConnectorSink: Send + Sync {
    type SinkResource: Send;

    async fn validate(&self) -> Result<(), CommonError>;

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError>;

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        resource: &mut Self::SinkResource,
    ) -> Result<(), CommonError>;

    async fn cleanup_sink(&self, _resource: Self::SinkResource) -> Result<(), CommonError> {
        Ok(())
    }
}
