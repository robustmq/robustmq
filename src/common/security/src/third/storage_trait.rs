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
use metadata_struct::auth::{acl::SecurityAcl, blacklist::SecurityBlackList, user::SecurityUser};

#[async_trait]
pub trait AuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<SecurityUser>, CommonError>;

    async fn read_all_acl(&self) -> Result<Vec<SecurityAcl>, CommonError>;

    async fn read_all_blacklist(&self) -> Result<Vec<SecurityBlackList>, CommonError>;
}
