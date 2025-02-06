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
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::sync::{LazyLock, Mutex, MutexGuard};

use std::hash::Hash;

pub type FamilyGauge<L> = Arc<RwLock<Family<L, Gauge>>>;

pub type FamilyCounter<L> = Arc<RwLock<Family<L, Counter>>>;

static REGISTRY: LazyLock<Mutex<Registry>> = LazyLock::new(|| Mutex::new(Registry::default()));

pub fn default() -> MutexGuard<'static, Registry> {
    REGISTRY.lock().unwrap()
}

#[macro_export]
macro_rules! register_gauge_metric {
    ($name:ident, $metric_name:expr, $help:expr,$label:ty) => {
        static $name: std::sync::LazyLock<common_base::metrics::registry::FamilyGauge<$label>> =
            std::sync::LazyLock::new(|| {
                common_base::metrics::registry::register_int_gauge_family($metric_name, $help)
            });
    };
}

#[macro_export]
macro_rules! register_counter_metric {
    ($name:ident, $metric_name:expr, $help:expr,$label:ty) => {
        static $name: std::sync::LazyLock<common_base::metrics::registry::FamilyCounter<$label>> =
            std::sync::LazyLock::new(|| {
                common_base::metrics::registry::register_int_counter_family($metric_name, $help)
            });
    };
}

#[macro_export]
macro_rules! gauge_metric_inc {
    ($family:ident,$label:ident) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(gauge) = family_r.get(&$label) {
                gauge.inc();
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
macro_rules! gauge_metric_inc_by {
    ($family:ident,$label:ident,$v:expr) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(gauge) = family_r.get(&$label) {
                gauge.inc_by($v);
                found = true;
            };
        }
        if !found {
            let family_w = family.write().unwrap();
            family_w.get_or_create(&$label).inc_by($v);
        }
    }};
}

#[macro_export]
macro_rules! gauge_metric_get {
    ($family:ident,$label:ident, $res:ident) => {{
        let family = $family.clone();
        let mut found = false;
        {
            let family_r = family.read().unwrap();
            if let Some(gauge) = family_r.get(&$label) {
                $res = gauge.get();
                found = true;
            };
        }
        if !found {
            let family_w = family.write().unwrap();
            $res = family_w.get_or_create(&$label).get();
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

/// Register a `Family<Gauge>` and wrap it in `Arc<RwLock<...>>`
pub fn register_int_gauge_family<L>(name: &str, help: &str) -> Arc<RwLock<Family<L, Gauge>>>
where
    L: EncodeLabelSet + Eq + Clone + Hash + Debug + Sync + Send + 'static,
{
    let family = Family::<L, Gauge>::default();
    default().register(name, help, family.clone());
    Arc::new(RwLock::new(family))
}

/// Register a `Family<Counter>` and wrap it in `Arc<RwLock<...>>`
pub fn register_int_counter_family<L>(name: &str, help: &str) -> Arc<RwLock<Family<L, Counter>>>
where
    L: EncodeLabelSet + Eq + Clone + Hash + Debug + Sync + Send + 'static,
{
    let family = Family::<L, Counter>::default();
    default().register(name, help, family.clone());
    Arc::new(RwLock::new(family))
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
    async fn test_gauge() {
        let family = register_int_gauge_family::<ClientConnectionLabels>(
            "client_connection",
            "client connection",
        );
        let mut tasks = vec![];
        for tid in 0..100 {
            let family = family.clone();
            let task = tokio::spawn(async move {
                let family = family.write().unwrap();
                let gauge = family.get_or_create(&ClientConnectionLabels {
                    client_id: format!("client-{}", tid),
                });
                gauge.inc();
            });
            tasks.push(task);
        }

        while let Some(task) = tasks.pop() {
            let _ = task.await;
        }

        let family = family.read().unwrap();

        let gauge = family.get_or_create(&ClientConnectionLabels {
            client_id: format!("client-{}", 0),
        });

        gauge.inc();
        assert_eq!(gauge.get(), 2);

        let mut buffer = String::new();

        let re = default();
        encode(&mut buffer, &re).unwrap();

        assert!(!buffer.is_empty());
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
                    client_id: format!("client-{}", tid),
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

        let re = default();
        encode(&mut buffer, &re).unwrap();

        assert!(!buffer.is_empty());
    }
}
