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

pub mod broker;
pub mod engine;
pub mod meta;
pub mod storage;

pub const PREFIX_META: &str = "/meta/";
pub const PREFIX_BROKER: &str = "/broker/";
pub const PREFIX_STORAGE: &str = "/storage/";
pub const PREFIX_ENGINE: &str = "/engine/";
pub const SEP: char = '/';
