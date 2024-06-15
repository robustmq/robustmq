use super::keys_prefix::{storeage_key_prefix_mqtt_user, storeage_key_prefix_share_sub_leader};

pub fn key_name_by_first_index() -> String {
    return "metasrv_first_index".to_string();
}

pub fn key_name_by_last_index() -> String {
    return "metasrv_last_index".to_string();
}

pub fn key_name_by_hard_state() -> String {
    return "metasrv_hard_state".to_string();
}

pub fn key_name_by_conf_state() -> String {
    return "metasrv_conf_state".to_string();
}

pub fn key_name_by_entry(idx: u64) -> String {
    return format!("metasrv_entry_{}", idx);
}

pub fn key_name_uncommit() -> String {
    return "metasrv_uncommit_index".to_string();
}

pub fn key_name_snapshot() -> String {
    return "metasrv_snapshot".to_string();
}

pub fn key_node(cluster_name: &String, node_id: u64) -> String {
    return format!("node_{}_{}", cluster_name, node_id);
}

pub fn key_cluster(cluster_name: &String) -> String {
    return format!("cluster_{}", cluster_name);
}

pub fn key_shard(cluster_name: &String, shard_name: String) -> String {
    return format!("shard_{}_{}", cluster_name, shard_name);
}

pub fn key_segment(cluster_name: &String, shard_name: &String, segement_seq: u64) -> String {
    return format!("segment_{}_{}_{}", cluster_name, shard_name, segement_seq);
}

pub fn key_all_cluster() -> String {
    return "all_cluster".to_string();
}

pub fn key_all_shard(cluster_name: &String) -> String {
    return format!("all_shard_{}", cluster_name);
}

pub fn uniq_key_id() -> String {
    return "generage_uniq_int_id".to_string();
}

pub fn share_sub_key(cluster_name: String, group_name: String) -> String {
    return format!(
        "{}/{}/{}",
        storeage_key_prefix_share_sub_leader(),
        cluster_name,
        group_name
    );
}

pub fn storage_key_mqtt_user(cluster_name: String, user_name: String) -> String {
    return format!(
        "{}/{}/{}",
        storeage_key_prefix_mqtt_user(),
        cluster_name,
        user_name
    );
}
