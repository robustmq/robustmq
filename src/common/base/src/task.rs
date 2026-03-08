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

#[derive(Clone)]
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
