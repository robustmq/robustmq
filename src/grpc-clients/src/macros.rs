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

/// Helper macro to implement the `RetriableRequest` trait for a given request type.
///
/// # Example
///
/// ```rust,ignore
/// impl_retriable_request!(Request, Client, Response, op, "Service", "Method");
/// impl_retriable_request!(Request, Client, Response, op, "Service", "Method", true);
/// ```
macro_rules! impl_retriable_request {
    ($req:ty, $client:ty, $res:ty, $op:ident, $service:expr, $method:expr) => {
        impl $crate::utils::RetriableRequest for $req {
            type Client = $client;
            type Response = $res;
            type Error = common_base::error::common::CommonError;

            fn method_name() -> &'static str {
                concat!($service, "/", $method)
            }

            fn get_client(pool: &$crate::pool::ClientPool, addr: &str) -> Self::Client {
                <$client>::new(pool.get_channel(addr))
            }

            async fn call_once(
                client: &mut Self::Client,
                request: Self,
            ) -> Result<Self::Response, Self::Error> {
                client
                    .$op(request)
                    .await
                    .map(|reply| reply.into_inner())
                    .map_err(Into::into)
            }
        }
    };

    ($req:ty, $client:ty, $res:ty, $op:ident, $service:expr, $method:expr, $is_write_request:expr) => {
        impl $crate::utils::RetriableRequest for $req {
            type Client = $client;
            type Response = $res;
            type Error = common_base::error::common::CommonError;

            const IS_WRITE_REQUEST: bool = $is_write_request;

            fn method_name() -> &'static str {
                concat!($service, "/", $method)
            }

            fn get_client(pool: &$crate::pool::ClientPool, addr: &str) -> Self::Client {
                <$client>::new(pool.get_channel(addr))
            }

            async fn call_once(
                client: &mut Self::Client,
                request: Self,
            ) -> Result<Self::Response, Self::Error> {
                client
                    .$op(request)
                    .await
                    .map(|reply| reply.into_inner())
                    .map_err(Into::into)
            }
        }
    };
}

pub(crate) use impl_retriable_request;
