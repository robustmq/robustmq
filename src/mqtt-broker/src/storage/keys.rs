pub fn lastwill_key(client_id: String) -> String {
    return format!("{}-{}", 0, client_id);
}

pub fn session_key(client_id: String) -> String {
    return format!("{}-{}", 1, client_id);
}

pub fn retain_message(topic_id: String) -> String {
    return format!("{}-{}", 2, topic_id);
}

pub fn topic_key(topic_id: String) -> String {
    return format!("{}-{}", 3, topic_id);
}

pub fn all_topic_key() -> String {
    return format!("{}-all-topic", 4);
}
