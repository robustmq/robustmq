use dashmap::DashMap;
use metadata_struct::mqtt::topic::Topic;

pub struct TopicManager {
    pub topic_list: DashMap<String, Topic>,
}

impl TopicManager {
    pub fn new() -> Self {
        TopicManager {
            topic_list: DashMap::with_capacity(2),
        }
    }

    // Cache
    pub fn add_topic(&self, topic_name: &str, topic: &Topic) {
        self.topic_list.insert(topic_name.to_owned(), topic.clone());
    }

    pub fn delete_topic(&self, topic_name: &str) {
        self.topic_list.remove(topic_name);
    }

    pub fn topic_exists(&self, topic_name: &str) -> bool {
        self.topic_list.contains_key(topic_name)
    }

    pub fn get_topic_by_name(&self, topic_name: &str) -> Option<Topic> {
        if let Some(topic) = self.topic_list.get(topic_name) {
            return Some(topic.clone());
        }
        None
    }

    pub fn get_all_topic_name(&self) -> Vec<String> {
        self.topic_list
            .iter()
            .map(|topic| topic.topic_name.clone())
            .collect()
    }

    pub fn get_storage_name_list_by_topic(&self, topic_name: &str) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(topic) = self.topic_list.get(topic_name) {
            for i in 0..topic.partition {
                results.push(self.build_storage_name(topic_name, i));
            }
        }
        results
    }

    pub fn build_storage_name(&self, topic_name: &str, partition: u32) -> String {
        format!("{}-{}", topic_name, partition)
    }
}
