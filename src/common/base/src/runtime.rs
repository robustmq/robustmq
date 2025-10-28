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

use std::sync::atomic::AtomicU32;

use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use tokio::runtime::{Builder, Runtime};

static GLOBAL_RUNTIME_ID: AtomicU32 = AtomicU32::new(0);
pub const THREAD_NAME_LABEL: &str = "thread_name";

lazy_static! {
    static ref METRIC_RUNTIME_THREADS_ALIVE: IntGaugeVec = register_int_gauge_vec!(
        "runtime_threads_alive",
        "runtime threads alive",
        &[THREAD_NAME_LABEL]
    )
    .unwrap();
    static ref METRIC_RUNTIME_THREADS_IDLE: IntGaugeVec = register_int_gauge_vec!(
        "runtime_threads_idle",
        "runtime threads idle",
        &[THREAD_NAME_LABEL]
    )
    .unwrap();
}

struct RuntimeBuilder {
    runtime_name: String,
    thread_name: String,
    builder: Builder,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self {
            runtime_name: format!(
                "runtime-{}",
                GLOBAL_RUNTIME_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ),
            thread_name: "default-worker".to_string(),
            builder: Builder::new_multi_thread(),
        }
    }
}

impl RuntimeBuilder {
    pub fn worker_threads(&mut self, val: usize) -> &mut Self {
        self.builder.worker_threads(val);
        self
    }

    #[allow(dead_code)]
    pub fn max_blocking_threads(&mut self, val: usize) -> &mut Self {
        self.builder.max_blocking_threads(val);
        self
    }

    pub fn runtime_name(&mut self, val: impl Into<String>) -> &mut Self {
        self.runtime_name = val.into();
        self
    }

    pub fn thread_name(&mut self, val: impl Into<String>) -> &mut Self {
        self.thread_name = val.into();
        self
    }

    pub fn build(&mut self) -> Runtime {
        let rt = self
            .builder
            .enable_all()
            .thread_name(self.thread_name.clone())
            .on_thread_start(on_thread_start(self.thread_name.clone()))
            .on_thread_stop(on_thread_stop(self.thread_name.clone()))
            .on_thread_park(on_thread_park(self.thread_name.clone()))
            .on_thread_unpark(on_thread_unpark(self.thread_name.clone()))
            .build()
            .unwrap();
        let _ = rt.enter();
        rt
    }
}

fn on_thread_start(thread_name: String) -> impl Fn() + 'static {
    move || {
        METRIC_RUNTIME_THREADS_ALIVE
            .with_label_values(&[thread_name.as_str()])
            .inc();
    }
}

fn on_thread_stop(thread_name: String) -> impl Fn() + 'static {
    move || {
        METRIC_RUNTIME_THREADS_ALIVE
            .with_label_values(&[thread_name.as_str()])
            .dec();
    }
}

fn on_thread_park(thread_name: String) -> impl Fn() + 'static {
    move || {
        METRIC_RUNTIME_THREADS_IDLE
            .with_label_values(&[thread_name.as_str()])
            .inc();
    }
}

fn on_thread_unpark(thread_name: String) -> impl Fn() + 'static {
    move || {
        METRIC_RUNTIME_THREADS_IDLE
            .with_label_values(&[thread_name.as_str()])
            .dec();
    }
}

pub fn create_runtime(runtime_name: &str, worker_threads: usize) -> Runtime {
    RuntimeBuilder::default()
        .runtime_name(runtime_name)
        .thread_name(runtime_name)
        .worker_threads(worker_threads)
        .build()
}

pub fn get_runtime_worker_threads() -> usize {
    64
}

#[cfg(test)]
mod tests {
    use std::thread::{self, sleep};
    use std::time::Duration;

    use super::create_runtime;

    #[test]
    fn test_metric() {
        let rt = create_runtime("test", 10);
        let _ = rt.enter();
        sleep(Duration::from_millis(500));

        let _hd = rt.spawn(async {
            sleep(Duration::from_millis(50));
        });

        sleep(Duration::from_millis(500));
        // let metric_text = dump_metrics().unwrap();
        // assert!(metric_text.contains("runtime_threads_idle{thread_name=\"test\"}"));
        // assert!(metric_text.contains("runtime_threads_alive{thread_name=\"test\"}"));
    }

    #[test]
    fn test_runtime_in_runtime() {
        let rt = create_runtime("test", 10);
        rt.spawn(async {
            println!("main spawn");
            tokio::spawn(async {
                println!("main spawn2");
                tokio::spawn(async {
                    println!("main spawn3");
                    tokio::spawn(async {
                        println!("main spawn4");
                    });
                });
            });
            sleep(Duration::from_millis(5));
        });

        sleep(Duration::from_secs(5));
    }

    #[test]
    fn test_get_cpu_num() {
        let num_threads = thread::available_parallelism().unwrap().get();
        println!("{}", num_threads);
    }
}
