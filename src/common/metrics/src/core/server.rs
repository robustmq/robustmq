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

use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::{LazyLock, Mutex, MutexGuard};

static REGISTRY: LazyLock<Mutex<Registry>> = LazyLock::new(|| Mutex::new(Registry::default()));

pub fn metrics_register_default() -> MutexGuard<'static, Registry> {
    REGISTRY.lock().unwrap()
}

pub fn dump_metrics() -> String {
    let mut buffer = String::new();
    let re = metrics_register_default();
    encode(&mut buffer, &re).unwrap();
    buffer
}

/// `NoLabelSet` is an empty label set type, used to build **unlabeled metrics**
/// Implemented `EncodeLabelSet` so that it can be correctly encoded as `{}` (empty labels) by Prometheus
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct NoLabelSet;

mod impl_encode_label_set {
    use prometheus_client::encoding::{EncodeLabelSet, LabelSetEncoder};
    use std::fmt::Error;

    use crate::core::server::NoLabelSet;

    impl EncodeLabelSet for NoLabelSet {
        fn encode(&self, _: LabelSetEncoder) -> Result<(), Error> {
            Ok(())
        }
    }
}
