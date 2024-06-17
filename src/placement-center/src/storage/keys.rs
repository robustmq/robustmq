pub fn key_name_by_first_index() -> String {
    return "/placement_center/raft/first_index".to_string();
}

pub fn key_name_by_last_index() -> String {
    return "/placement_center/raft/last_index".to_string();
}

pub fn key_name_by_hard_state() -> String {
    return "/placement_center/raft/hard_state".to_string();
}

pub fn key_name_by_conf_state() -> String {
    return "/placement_center/raft/conf_state".to_string();
}

pub fn key_name_by_entry(idx: u64) -> String {
    return format!("/placement_center/raft/entry/{}", idx);
}

pub fn key_name_uncommit() -> String {
    return "/placement_center/raft/uncommit_index".to_string();
}

pub fn key_name_snapshot() -> String {
    return "/placement_center/raft/snapshot".to_string();
}

pub fn key_node(cluster_name: &String, node_id: u64) -> String {
    return format!(
        "/placement_center/cluster/{}/node/{}",
        cluster_name, node_id
    );
}

pub fn key_cluster(cluster_name: &String) -> String {
    return format!("/placement_center/cluster/{}", cluster_name);
}

pub fn key_shard(cluster_name: &String, shard_name: String) -> String {
    return format!("/journal/{}/shard/{}", cluster_name, shard_name);
}

pub fn key_segment(cluster_name: &String, shard_name: &String, segement_seq: u64) -> String {
    return format!(
        "/journal/{}/segment/{}/{}",
        cluster_name, shard_name, segement_seq
    );
}

pub fn storage_key_mqtt_user(cluster_name: String, user_name: String) -> String {
    return format!("/mqtt/{}/user/{}", cluster_name, user_name);
}

pub fn storage_key_mqtt_user_cluster_prefix(cluster_name: String) -> String {
    return format!("/mqtt/{}/user/", cluster_name);
}

pub fn storage_key_mqtt_topic(cluster_name: String, user_name: String) -> String {
    return format!("/mqtt/{}/topic/{}", cluster_name, user_name);
}

pub fn storage_key_mqtt_topic_cluster_prefix(cluster_name: String) -> String {
    return format!("/mqtt/{}/topic/", cluster_name);
}
