use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Node {
    pub mqtt4_enable: bool,
    pub mqtt5_enable: bool,
}
