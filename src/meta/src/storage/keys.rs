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
