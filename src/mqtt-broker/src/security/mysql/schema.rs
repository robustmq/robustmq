// Copyright 2023 RobustMQ Team
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




#[derive(Debug, PartialEq, Eq)]
pub struct TAuthUser {
    pub id: u64,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub is_superuser: String,
    pub created: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TAuthAcl {
    pub id: u64,
    pub allow: u64,
    pub ipaddr: String,
    pub username: String,
    pub clientid: String,
    pub access: u64,
    pub topic: String,
}
