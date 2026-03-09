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

use dashmap::DashMap;
use tokio::task::JoinHandle;
use tracing::{error, info};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TaskKind {
    BrokerNodeCall,
    DelayMessagePop,
}

impl std::fmt::Display for TaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskKind::BrokerNodeCall => write!(f, "BrokerNodeCall"),
            TaskKind::DelayMessagePop => write!(f, "DelayMessagePop"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskState {
    Running,
    Stopped,
    Failed(String),
}

#[derive(Default, Clone)]
pub struct TaskSupervisor {
    task_status: DashMap<String, TaskState>,
}

impl TaskSupervisor {
    pub fn new() -> Self {
        TaskSupervisor {
            task_status: DashMap::with_capacity(2),
        }
    }

    pub fn spawn<F>(&self, kind: String, fut: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let sup = self.clone();
        let task_name = kind.to_string();
        tokio::task::Builder::new()
            .name(&task_name)
            .spawn(async move {
                sup.set_state(kind.clone(), TaskState::Running).await;
                info!("Task {} started", kind);
                let inner = tokio::task::Builder::new()
                    .name(&format!("{kind}/inner"))
                    .spawn(fut)
                    .expect("failed to spawn inner task");
                match inner.await {
                    Ok(()) => {
                        info!("Task {} stopped normally", kind);
                        sup.set_state(kind.clone(), TaskState::Stopped).await;
                    }
                    Err(e) => {
                        error!("Task {} failed: join error: {}", kind, e);
                        sup.set_state(kind.clone(), TaskState::Failed(format!("join error: {e}")))
                            .await;
                    }
                }
            })
            .expect("failed to spawn task")
    }

    pub fn ready(self, kind: &str) -> bool {
        if let Some(state) = self.task_status.get(kind) {
            return *state == TaskState::Running;
        }
        false
    }

    async fn set_state(&self, kind: String, state: TaskState) {
        self.task_status.insert(kind, state);
    }
}
