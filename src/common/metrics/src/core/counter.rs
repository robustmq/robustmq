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
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::core::server::metrics_register_default;

pub type FamilyCounter<L> = Arc<RwLock<Family<L, Counter>>>;

// https://docs.rs/prometheus-client/0.23.1/prometheus_client/index.html

// Counter
#[macro_export]
macro_rules! register_counter_metric {
    ($name:ident, $metric_name:expr, $help:expr,$label:ty) => {
        static $name: std::sync::LazyLock<$crate::core::counter::FamilyCounter<$label>> =
            std::sync::LazyLock::new(|| {
                $crate::core::counter::register_int_counter_family($metric_name, $help)
            });
    };
}

pub fn register_int_counter_family<L>(name: &str, help: &str) -> Arc<RwLock<Family<L, Counter>>>
where
    L: EncodeLabelSet + Eq + Clone + Hash + Debug + Sync + Send + 'static,
{
    let family = Family::<L, Counter>::default();
    metrics_register_default().register(name, help, family.clone());
    Arc::new(RwLock::new(family))
}

#[macro_export]
macro_rules! counter_metric_inc {
    ($family:ident,$label:ident) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(counter) = family_r.get(&$label) {
                counter.inc();
                found = true;
            };
        }
        if !found {
            let family_w = family.write().unwrap();
            family_w.get_or_create(&$label).inc();
        }
    }};
}

#[macro_export]
macro_rules! counter_metric_inc_by {
    ($family:ident,$label:ident, $value:expr) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(counter) = family_r.get(&$label) {
                counter.inc_by($value);
                found = true;
            };
        }
        if !found {
            let family_w = family.write().unwrap();
            family_w.get_or_create(&$label).inc_by($value);
        }
    }};
}

#[macro_export]
macro_rules! counter_metric_get {
    ($family:ident,$label:ident, $res:ident) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(counter) = family_r.get(&$label) {
                $res = counter.get();
                found = true;
            };
        }
        if !found {
            let family_w = family.write().unwrap();
            $res = family_w.get_or_create(&$label).get();
        }
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use prometheus_client::encoding::text::encode;

    #[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
    struct ClientConnectionLabels {
        client_id: String,
    }

    #[tokio::test]
    async fn test_counter() {
        let family = register_int_counter_family::<ClientConnectionLabels>(
            "client_connection",
            "client connection",
        );
        let mut tasks = vec![];
        for tid in 0..100 {
            let family = family.clone();
            let task = tokio::spawn(async move {
                let family = family.write().unwrap();
                let counter = family.get_or_create(&ClientConnectionLabels {
                    client_id: format!("client-{tid}"),
                });
                counter.inc();
            });
            tasks.push(task);
        }

        // 逐个 `await` 任务
        while let Some(task) = tasks.pop() {
            let _ = task.await;
        }

        let mut buffer = String::new();

        let re = metrics_register_default();
        encode(&mut buffer, &re).unwrap();

        assert!(!buffer.is_empty());
    }
}
