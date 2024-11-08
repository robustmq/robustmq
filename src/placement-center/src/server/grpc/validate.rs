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
use protocol::placement_center::placement_center_inner::{ClusterType, DeleteIdempotentDataRequest, RegisterNodeRequest, SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest};
use regex::Regex;
use tonic::Status;

pub trait ValidateExt {
    fn validate_ext(&self) -> Result<(), Status>;
}

fn ensure_param_not_empty(param: &str) -> Result<(), Status> {
    if param.is_empty() {
        Err(
            Status::invalid_argument(
                CommonError::ParameterCannotBeNull(param.to_string()).to_string()
            )
        )
    } else {
        Ok(())
    }
}

fn validate_ip(field: &str, ip: &str) -> Result<(), Status> {
    let ipv4_re = Regex::new(r"^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$");
    if ipv4_re.unwrap().is_match(ip) {
        Ok(())
    }else {
        Err(Status::invalid_argument(
            CommonError::InvalidParameterFormat(field.to_string()).to_string()
        ))
    }
}

impl ValidateExt for RegisterNodeRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if !ClusterType::is_valid(self.cluster_type) {
            return Err(Status::unavailable(
                CommonError::UnavailableClusterType.to_string(),
            ));
        }

        ensure_param_not_empty(&self.cluster_name)?;
        ensure_param_not_empty(&self.node_ip)?;
        ensure_param_not_empty(&self.node_inner_addr)?;

        validate_ip("node_ip", &self.node_ip)?;
        validate_ip("node_inner_addr", &self.node_inner_addr)?;


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

impl ValidateExt for SetIdempotentDataRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if self.cluster_name.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("cluster name".to_string()).to_string(),
            ));
        }

        if self.producer_id.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("producer id".to_string()).to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod validate_test {
    use super::*;

    #[test]
    fn test_ensure_param_not_empty() {
        let param = String::new();
        let result = ensure_param_not_empty(&param);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message(), Status::invalid_argument(
            CommonError::ParameterCannotBeNull(param.to_string()).to_string()
        ).message());

        let param = "cluster_name";
        let result = ensure_param_not_empty(&param);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ip() {
        let test_ips = [
            "192.168.1.1",
            "256.256.256.256",
            "10.0.0.1",
            "127.0.0.1",
        ];

        for ip in test_ips {
            let result = validate_ip("node_ip", ip);
            match ip {
                "256.256.256.256" => {
                    println!("{:?}", result);
                    assert!(result.is_err());
                    let status = result.unwrap_err();
                    assert_eq!(status.code(), Status::invalid_argument("node_ip".to_string()).code());
                    assert_eq!(status.message(), Status::invalid_argument(
                        CommonError::InvalidParameterFormat("node_ip".to_string()).to_string()
                    ).message());
                }
                &_ => {
                    assert!(result.is_ok());
                }
            }
        }
    }
}