#[derive(Debug, Clone)]
pub struct NatsSubscriber {
    pub uniq_id: String,
    pub tenant: String,
    pub connect_id: u64,
    pub sid: String,
    /// Original subscription subject pattern (may contain wildcards).
    pub sub_subject: String,
    /// Concrete subject name matched against sub_subject.
    pub subject: String,
    pub broker_id: u64,
    /// Non-empty for queue-group subscriptions.
    pub queue_group: String,
    pub create_time: u64,
}
