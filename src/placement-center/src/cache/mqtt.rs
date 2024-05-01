use dashmap::DashMap;
use crate::structs::share_sub::ShareSub;

pub struct MqttCache {
    pub share_sub_list: DashMap<String, ShareSub>,
}

impl MqttCache {
    pub fn new() -> MqttCache {
        return MqttCache {
            share_sub_list: DashMap::with_capacity(128),
        };
    }

    pub fn add_share_sub(&self, cluster_name: String, group_id: String, share_sub: ShareSub) {
        let key = self.key_name(cluster_name, group_id);
        self.share_sub_list.insert(key, share_sub);
    }

    pub fn remove_share_sub(&self, cluster_name: String, group_id: String) {
        let key = self.key_name(cluster_name, group_id);
        self.share_sub_list.remove(&key);
    }

    pub fn get_share_sub(&self, cluster_name: String, group_id: String) -> Option<ShareSub>{
        let key = self.key_name(cluster_name, group_id);
        if let Some(value) = self.share_sub_list.get(&key){
            return Some(value.clone())
        }
        return None;
    }

    pub fn key_name(&self, cluster_name: String, group_id: String) -> String {
        return format!("{}-{}", cluster_name, group_id);
    }
}
