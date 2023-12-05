/*
 * Copyright (c) 2023 RobustMQ Team 
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *     http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


 pub async fn metrics_handler() -> String {
    // return crate::SERVER_METRICS.gather();
    return String::from("")
}

pub async fn welcome_handler() -> &'static str {
    // crate::SERVER_METRICS.set_server_status_stop();
    "Welcome to RobustMQ"
}