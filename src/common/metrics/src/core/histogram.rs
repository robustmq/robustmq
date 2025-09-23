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
use prometheus_client::metrics::family::{Family, MetricConstructor};
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::core::server::metrics_register_default;

pub type FamilyHistogram<L> = Arc<RwLock<Family<L, Histogram, BucketType>>>;

/// BucketType defines the behavior of passing buckets when building a Histogram
///
/// this is usually used when building internal components
#[derive(Clone)]
pub enum BucketType {
    /// Explicitly pass `[f64]` bucket type.
    /// Because metrics typically persist throughout the life of the application, `&'static` is allowed here
    ///
    /// example:
    /// ```
    /// # use common_base::register_histogram_metric!
    /// register_histogram_metric!(
    ///     FOO,
    ///     "foo.metrics",
    ///     "foo metrics help",
    ///     FooLabel,
    ///     [1.0 ,2.0, 3.0]
    /// );
    /// ```
    PlaintextBucket(&'static [f64]),

    /// Generate the corresponding buckets through the index constructor.
    /// For parameter definition and function definition, see [`exponential_buckets`]
    ///
    /// example:
    /// ```
    /// # use common_base::register_histogram_metric!
    /// register_histogram_metric!(
    ///     FOO,
    ///     "foo.metrics",
    ///     "foo metrics help",
    ///     FooLabel,
    ///     1.0,
    ///     2.0,
    ///     12,
    /// );
    /// ```
    ExponentialBuckets {
        start: f64,
        factor: f64,
        length: u16,
    },
}

impl MetricConstructor<Histogram> for BucketType {
    fn new_metric(&self) -> Histogram {
        match *self {
            BucketType::PlaintextBucket(items) => Histogram::new(items.iter().copied()),
            BucketType::ExponentialBuckets {
                start,
                factor,
                length,
            } => Histogram::new(exponential_buckets(start, factor, length)),
        }
    }
}

#[macro_export]
macro_rules! register_histogram_metric {
    ($name:ident, $metric_name:expr, $help:expr, $label:ty, [$($val:expr),* $(,)?]) => {
        const BUCKETS: &[f64] = &[$($val as f64),*];
        static $name: std::sync::LazyLock<
            $crate::core::histogram::FamilyHistogram<$label>,
        > = std::sync::LazyLock::new(|| {
            $crate::core::histogram::register_histogram_family(
                $metric_name,
                $help,
                $crate::core::histogram::BucketType::PlaintextBucket(BUCKETS),
            )
        });

    };
    ($name:ident, $metric_name:expr, $help:expr, $label:ty, $start:expr, $factor:expr, $length:expr) => {
        static $name: std::sync::LazyLock<
            $crate::core::histogram::FamilyHistogram<$label>,
        > = std::sync::LazyLock::new(|| {
            $crate::core::histogram::register_histogram_family(
                $metric_name,
                $help,
                $crate::core::histogram::BucketType::ExponentialBuckets{
                    start: $start,
                    factor: $factor,
                    length: $length,
                },
            )
        });
    };
}

/// Register a histogram metric with default request duration buckets
/// This macro uses the predefined DEFAULT_REQUEST_DURATION_BUCKETS configuration
///
/// # Example
/// ```
/// register_histogram_metric_with_ms_default_buckets!(
///     HTTP_REQUEST_DURATION,
///     "http_request_duration_ms",
///     "Duration of HTTP requests in milliseconds",
///     HttpLabel
/// );
/// ```
#[macro_export]
macro_rules! register_histogram_metric_ms_with_default_buckets {
    ($name:ident, $metric_name:expr, $help:expr, $label:ty) => {
        static $name: std::sync::LazyLock<$crate::core::histogram::FamilyHistogram<$label>> =
            std::sync::LazyLock::new(|| {
                $crate::core::histogram::register_histogram_family(
                    $metric_name,
                    $help,
                    $crate::core::histogram::DEFAULT_REQUEST_DURATION_BUCKETS,
                )
            });
    };
}

pub fn register_histogram_family<L>(
    name: &str,
    help: &str,
    bucket_type: BucketType,
) -> Arc<RwLock<Family<L, Histogram, BucketType>>>
where
    L: EncodeLabelSet + Eq + Clone + Hash + Debug + Sync + Send + 'static,
{
    let family = Family::<L, Histogram, BucketType>::new_with_constructor(bucket_type);

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

/// Default bucket configuration for request duration metrics (exponential buckets)
/// start=1.0, factor=2.0, length=10 generates buckets: [1, 2, 4, 8, 16, 32, 64, 128, 256, 512]
pub const DEFAULT_REQUEST_DURATION_BUCKETS: BucketType = BucketType::ExponentialBuckets {
    start: 1.0,
    factor: 2.0,
    length: 20,
};

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use prometheus_client::encoding::EncodeLabelSet;

    #[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
    struct Labels {
        uri: String,
    }

    register_histogram_metric!(
        HTTP_DURATION_PLAINTEXT_BUCKETS,
        "http.duration.plaintext",
        "duration ms of http call(plaintext mode)",
        Labels,
        1.0,
        2.0,
        10
    );

    register_histogram_metric!(
        HTTP_DURATION_EXPONENTIAL_BUCKETS,
        "http.duration.exponential",
        "duration ms of http call(exponential mode)",
        Labels,
        [1.0, 2.0, 3.0]
    );

    #[tokio::test]
    async fn test_histogram_plaintext() {
        let labels = Labels {
            uri: String::from("/get"),
        };
        let ms: f64 = 100.0;
        histogram_metric_observe!(HTTP_DURATION_PLAINTEXT_BUCKETS, ms, labels);
    }

    #[tokio::test]
    async fn test_histogram_exponential() {
        let labels = Labels {
            uri: String::from("/set"),
        };
        let ms: f64 = 101.0;
        histogram_metric_observe!(HTTP_DURATION_EXPONENTIAL_BUCKETS, ms, labels);
    }
}
