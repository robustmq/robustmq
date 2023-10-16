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

use server::ServerMetrics;
use lazy_static::lazy_static;
mod server;

lazy_static! {
    pub static ref SERVER_METRICS:ServerMetrics = server_metrics();
}

/// Start Metrics Register
pub fn server_metrics() -> ServerMetrics{
    let server_metrics: ServerMetrics = ServerMetrics::new();
    server_metrics.register_metrics();
    return server_metrics;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
