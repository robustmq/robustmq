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

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use std::hash::Hash;

use super::metrics_register_default;

pub type FamilyHistogram<L> = Arc<RwLock<Family<L, Histogram>>>;

// Histogram
#[macro_export]
macro_rules! register_histogram_metric {
    ($name:ident, $metric_name:expr, $help:expr,$label:ty) => {
        static $name: std::sync::LazyLock<
            common_base::metrics::histogram::FamilyHistogram<$label>,
        > = std::sync::LazyLock::new(|| {
            common_base::metrics::histogram::register_histogram_family($metric_name, $help)
        });
    };
}

pub fn register_histogram_family<L>(name: &str, help: &str) -> Arc<RwLock<Family<L, Histogram>>>
where
    L: EncodeLabelSet + Eq + Clone + Hash + Debug + Sync + Send + 'static,
{
    let family = Family::<L, Histogram>::new_with_constructor(|| {
        Histogram::new(exponential_buckets(1.0, 2.0, 10))
    });

    metrics_register_default().register(name, help, family.clone());
    Arc::new(RwLock::new(family))
}

#[macro_export]
macro_rules! histogram_metric_observe {
    ($family:ident, $value:ident, $label:ident) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(histogram) = family_r.get(&$label) {
                histogram.observe($value);
                found = true;
            };
        }
        if !found {
            let family_w = family.write().unwrap();
            family_w.get_or_create(&$label).observe($value);
        }
    }};
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_histogram() {}
}
