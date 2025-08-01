#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    /// Not started
    NotStarted,
    /// Starting
    Starting,
    /// Running
    Running,
    /// Stopping
    Stopping,
}
