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

pub fn histogram_default_ms() -> Vec<f64> {
    vec![
        1.0, 5.0, 10.0, 30.0, 50.0, 100.0, 200.0, 500.0, 700.0, 1000.0, 1500.0, 2000.0, 5000.0,
        10000.0, 200000.0, 30000.0, 60000.0,
    ]
}
