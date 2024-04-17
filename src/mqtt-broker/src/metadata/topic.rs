use common_base::tools::{now_mills, unique_id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Topic {
    pub topic_id: String,
    pub topic_name: String,
    pub create_time: u128,
}

impl Topic {
    pub fn new(topic_name: &String) -> Self {
        return Topic {
            topic_id: unique_id(),
            topic_name: topic_name.clone(),
            create_time: now_mills(),
        };
    }
}
