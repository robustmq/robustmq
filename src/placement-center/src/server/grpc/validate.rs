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
use protocol::placement_center::placement_center_inner::{
    ClusterType, DeleteIdempotentDataRequest, GetResourceConfigRequest, RegisterNodeRequest,
    SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};
use protocol::placement_center::placement_center_mqtt::GetShareSubLeaderRequest;
use tonic::Status;

pub trait ValidateExt {
    fn validate_ext(&self) -> Result<(), Status>;
}

//Ensure that the param is not empty.
//
//Parameters:
// - `field_name: &str`: The name of the field.
// - `field_value: &str`: The value of the field.
//
//Returns: Result<(), Status>.
fn ensure_param_not_empty(field_name: &str, field_value: &str) -> Result<(), Status> {
    if field_value.is_empty() {
        Err(Status::invalid_argument(
            CommonError::ParameterCannotBeNull(field_name.to_string()).to_string(),
        ))
    } else {
        Ok(())
    }
}

//This method is used to verify whether the IP address is correct,
// and it can also validate cases with port numbers.
//
//Parameters:
// - `field_name: &str`: The name of the field.
// - `ip: &str`: ip addr.
//
//Returns: Result<(), Status>.
fn validate_ip(field_name: &str, ip: &str) -> Result<(), Status> {
    let is_ip_addr_correct: Result<IpAddr, _> = ip.parse();
    let is_socket_correct: Result<SocketAddr, _> = ip.parse();

    if is_ip_addr_correct.is_ok() || is_socket_correct.is_ok() {
        Ok(())
    } else {
        Err(Status::invalid_argument(
            CommonError::InvalidParameterFormat(field_name.to_string(), ip.to_string()).to_string(),
        ))
    }
}

#[allow(dead_code)]
fn is_valid_cluster_type(cluster_type: i32) -> Result<(), Status> {
    if !ClusterType::is_valid(cluster_type) {
        Err(Status::invalid_argument(
            CommonError::UnavailableClusterType.to_string(),
        ))
    } else {
        Ok(())
    }
}

impl ValidateExt for RegisterNodeRequest {
    // validate params:
    // 1. is valid cluster_type
    // 2. cluster_name, node_ip, node_inner_addr can not empty
    // 3. node_ip, node_inner_add is correct ipv4/v6
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
        ensure_param_not_empty("cluster_name", &self.cluster_name)?;

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

impl ValidateExt for GetResourceConfigRequest {
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
        ensure_param_not_empty("cluster_name", &self.cluster_name)?;
        ensure_param_not_empty("producer_id", &self.producer_id)?;

        Ok(())
    }
}

impl ValidateExt for GetShareSubLeaderRequest {
    fn validate_ext(&self) -> Result<(), Status> {
        ensure_param_not_empty("group_name", &self.group_name)?;
        ensure_param_not_empty("cluster_name", &self.cluster_name)?;

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
        assert_eq!(
            result.unwrap_err().message(),
            Status::invalid_argument(
                CommonError::ParameterCannotBeNull("cluster_name".to_string()).to_string()
            )
            .message()
        );
        let result = ensure_param_not_empty("cluster_name", "cluster_name");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ip() {
        let test_ips = [
            "192.168.1.1",
            "256.256.256.256",
            "10.0.0.1",
            "127.0.0.1",
            "127.0.0.1:8228",
        ];

        for ip in test_ips {
            let result = validate_ip("node_ip", ip);
            match ip {
                "256.256.256.256" => {
                    println!("{:?}", result);
                    assert!(result.is_err());
                    let status = result.unwrap_err();
                    assert_eq!(
                        status.code(),
                        Status::invalid_argument("node_ip".to_string()).code()
                    );
                    assert_eq!(
                        status.message(),
                        Status::invalid_argument(
                            CommonError::InvalidParameterFormat(
                                "node_ip".to_string(),
                                ip.to_string()
                            )
                            .to_string()
                        )
                        .message()
                    );
                }
                &_ => {
                    println!("{:?}", ip);
                    assert!(result.is_ok());
                }
            }
        }
    }

    #[test]
    fn test_valid_cluster_type() {
        assert!(is_valid_cluster_type(1).is_ok());
        let result = is_valid_cluster_type(10);
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(
            status.message(),
            Status::invalid_argument(CommonError::UnavailableClusterType.to_string(),).message()
        )
    }
}
