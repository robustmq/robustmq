use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize, Deserialize)]
pub struct Subscriber {
    pub sub_name: String,
}
