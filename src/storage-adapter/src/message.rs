use common_base::tools::now_mills;

#[derive(Clone)]
pub struct HeaderProperties {
    pub name: String,
    pub value: String,
}

#[derive(Clone)]
pub struct Message {
    pub key: String,
    pub data: Vec<u8>,
    pub create_time: u128,
    pub properties: Vec<HeaderProperties>,
}

impl Message {
    pub fn build(key: Option<String>, data: Vec<u8>, properties: Vec<HeaderProperties>) -> Self {
        let key_val = if let Some(k) = key { k } else { "".to_string() };
        return Message {
            key: key_val,
            data,
            create_time: now_mills(),
            properties,
        };
    }
}
