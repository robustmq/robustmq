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

use std::net::{IpAddr, SocketAddr};
use common_base::error::common::CommonError;
use protocol::placement_center::placement_center_inner::{ClusterType, DeleteIdempotentDataRequest, RegisterNodeRequest, SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest};
use tonic::Status;

pub trait ValidateExt {
    fn validate_ext(&self) -> Result<(), Status>;
}

fn ensure_param_not_empty(field_name: &str, field_value: &str) -> Result<(), Status> {
    if field_value.is_empty() {
        println!("field_name: {:?}, field_value: {:?}", field_name, field_value);
        Err(
            Status::invalid_argument(
                CommonError::ParameterCannotBeNull(field_name.to_string()).to_string()
            )
        )
    } else {
        println!("field_name: {:?}, field_value: {:?}", field_name, field_value);
        Ok(())
    }
}

fn validate_ip(field_name: &str, ip: &str) -> Result<(), Status> {
    let is_ip_addr_correct:Result<IpAddr, _> = ip.parse();
    let is_socket_correct: Result<SocketAddr, _> = ip.parse();

    if is_ip_addr_correct.is_ok() || is_socket_correct.is_ok() {
        Ok(())
    }else {
        Err(Status::invalid_argument(
            CommonError::InvalidParameterFormat(field_name.to_string(), ip.to_string()).to_string()
        ))
    }
}

impl ValidateExt for RegisterNodeRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        if !ClusterType::is_valid(self.cluster_type) {
            return Err(Status::invalid_argument(
                CommonError::UnavailableClusterType.to_string(),
            ));
        }

        ensure_param_not_empty("cluster_name", &self.cluster_name)?;
        ensure_param_not_empty("node_ip", &self.node_ip)?;
        ensure_param_not_empty("node_inner_addr", &self.node_inner_addr)?;

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
        let result = ensure_param_not_empty("cluster_name", &param);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message(), Status::invalid_argument(
            CommonError::ParameterCannotBeNull("cluster_name".to_string()).to_string()
        ).message());

        let param = "cluster_name";
        let result = ensure_param_not_empty("cluster_name", &param);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ip() {
        let test_ips = [
            "192.168.1.1",
            "256.256.256.256",
            "10.0.0.1",
            "127.0.0.1",
            "127.0.0.1:8228"
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
                        CommonError::InvalidParameterFormat("node_ip".to_string(), ip.to_string()).to_string()
                    ).message());
                }
                &_ => {
                    println!("{:?}", ip);
                    assert!(result.is_ok());
                }
            }
        }
    }
}