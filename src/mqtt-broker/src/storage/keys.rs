pub fn retain_message(topic_id: String) -> String {
    return format!("{}-{}", 2, topic_id);
}

pub fn cluster_config_key() -> String {
    return format!("{}-cluster-config", 5);
}

