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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TaskKind {
    NetworkGc,
    OffsetFlush,
    RuntimeMonitor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskState {
    Running,
    Stopped,
    Failed(String),
}

#[derive(Default, Clone)]
pub struct TaskSupervisor {
    task_status: DashMap<TaskKind, TaskState>,
}

impl TaskSupervisor {
    pub fn new() -> Self {
        TaskSupervisor {
            task_status: DashMap::with_capacity(2),
        }
    }

    pub fn spawn<F>(&self, kind: TaskKind, fut: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let sup = self.clone();
        tokio::spawn(async move {
            sup.set_state(kind, TaskState::Running).await;
            let inner = tokio::spawn(fut);
            match inner.await {
                Ok(()) => sup.set_state(kind, TaskState::Stopped).await,
                Err(e) => {
                    sup.set_state(kind, TaskState::Failed(format!("join error: {e}")))
                        .await
                }
            }
        })
    }

    pub fn ready(self, kind: &TaskKind) -> bool {
        if let Some(state) = self.task_status.get(kind) {
            return *state == TaskState::Running;
        }
        false
    }

    async fn set_state(&self, kind: TaskKind, state: TaskState) {
        self.task_status.insert(kind, state);
    }
}
