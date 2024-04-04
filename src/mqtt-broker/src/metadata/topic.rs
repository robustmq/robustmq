use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize, Deserialize)]
pub struct Topic {
    pub topic_id: String,
    pub topic_name: String,
}

impl Topic{
   
}
