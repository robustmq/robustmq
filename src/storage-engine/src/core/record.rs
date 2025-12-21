use bytes::Bytes;
use common_base::tools::now_second;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug)]
pub struct StorageEngineRecordMetadata {
    pub offset: u64,
    pub shard: String,
    pub segment: u32,
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub create_t: u64,
}

impl StorageEngineRecordMetadata {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }

    pub fn new(
        offset: u64,
        shard: &str,
        segment: u32,
        key: &Option<String>,
        tags: &Option<Vec<String>>,
    ) -> Self {
        StorageEngineRecordMetadata {
            offset,
            shard: shard.to_string(),
            segment,
            key: key.clone(),
            tags: tags.clone(),
            create_t: now_second(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageEngineRecord {
    pub metadata: StorageEngineRecordMetadata,
    pub data: Bytes,
}

impl StorageEngineRecord {
    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }
}
