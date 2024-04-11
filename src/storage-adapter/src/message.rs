#[derive(Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone)]
pub struct Message {
    pub header: Option<Vec<Header>>,
    pub key: Option<String>,
    pub data: Vec<u8>,
    pub create_time: Option<u128>,
}

impl Message {
    pub fn build_a(
        key: Option<String>,
        data: Vec<u8>,
        header: Option<Vec<Header>>,
        create_time: Option<u128>,
    ) -> Self {
        return Message {
            key,
            data,
            create_time,
            header,
        };
    }

    pub fn build_b(data: Vec<u8>) -> Self {
        return Message {
            key: None,
            data,
            create_time: None,
            header: None,
        };
    }

    pub fn build_c(key: String, data: Vec<u8>) -> Self {
        return Message {
            key: Some(key),
            data,
            create_time: None,
            header: None,
        };
    }

    pub fn build_d(key: String, header: Vec<Header>, data: Vec<u8>) -> Self {
        return Message {
            key: Some(key),
            data,
            create_time: None,
            header: Some(header),
        };
    }

    pub fn build_e(data: String) -> Self {
        return Message {
            key: None,
            data: data.as_bytes().to_vec(),
            create_time: None,
            header: None,
        };
    }
}
