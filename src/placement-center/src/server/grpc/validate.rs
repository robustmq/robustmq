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

use common_base::error::common::CommonError;
use protocol::placement_center::placement_center_inner::{
    DeleteIdempotentDataRequest, RegisterNodeRequest, SetResourceConfigRequest,
    UnRegisterNodeRequest,
};
use tonic::Status;

pub trait ValidateExt {
    fn validate_ext(&self) -> Result<(), Status>;
}

impl ValidateExt for RegisterNodeRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if self.cluster_name.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("cluster name".to_string()).to_string(),
            ));
        }

        if self.node_ip.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("node ip".to_string()).to_string(),
            ));
        }
        Ok(())
    }
}

impl ValidateExt for UnRegisterNodeRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if self.cluster_name.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("cluster name".to_string()).to_string(),
            ));
        }
        Ok(())
    }
}

impl ValidateExt for DeleteIdempotentDataRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if self.cluster_name.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("cluster name".to_string()).to_string(),
            ));
        }
        Ok(())
    }
}

impl ValidateExt for SetResourceConfigRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if self.cluster_name.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("cluster name".to_string()).to_string(),
            ));
        }
        Ok(())
    }
}
