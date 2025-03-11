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

pub fn bool_to_vec(b: bool) -> Vec<u8> {
    if b {
        vec![0x01]
    } else {
        vec![0x00]
    }
}

pub fn vec_to_bool(v: &Vec<u8>) -> bool {
    matches!(v.as_slice(), [0x01])
}
