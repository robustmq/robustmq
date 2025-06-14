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

use prost_validate::Validator;

pub struct FooService;

#[derive(prost_validate::Validator)]
pub struct Request {
    #[validate(r#type(string(r#const = "foo")))]
    name: String,
}

pub struct Response;

#[async_trait::async_trait]
pub trait HeyGRPC {
    async fn hey(
        &self,
        _request: tonic::Request<Request>,
    ) -> Result<tonic::Response<Response>, tonic::Status>;
}

#[async_trait::async_trait]
impl HeyGRPC for FooService {
    #[robustmq_macro::validate_req(validator = prost_validate::Validator)]
    async fn hey(
        &self,
        _request: tonic::Request<Request>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        Ok(tonic::Response::new(Response {}))
    }
}

fn main() {}
