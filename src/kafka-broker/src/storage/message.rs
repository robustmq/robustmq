use bytes::{Bytes};
use kafka_protocol::indexmap::IndexMap;
use kafka_protocol::protocol::StrBytes;
use common_base::utils::crc::calc_crc32;
use kafka_protocol::records as kp;
use metadata_struct::adapter::record::{Header, Record};

pub struct TopicPartition {
    pub topic: String,
    pub partition: i32,
}

pub struct Message {
    pub topic_partition: TopicPartition,
    pub offset: i64,
    pub record: Record,
}

impl Message {
    pub fn encode(r: kp::Record) -> Record {
        let key = r.key.map(|b| String::from_utf8_lossy(&b).to_string());
        let offset = (r.offset >= 0).then(|| r.offset as u64);
        let timestamp = r.timestamp.max(0) as u64;

        let header = {
            let headers = r
                .headers
                .into_iter()
                .filter_map(|(name, bytes)| {
                    let name = String::from_utf8_lossy(name.as_ref()).to_string();
                    bytes.map(|v| Header {
                        name,
                        value: String::from_utf8_lossy(&v).to_string(),
                    })
                })
                .collect::<Vec<Header>>();
            Some(headers)
        };
        let data = r.value.unwrap_or_default();

        Record {
            offset,
            header,
            key,
            data,
            tags: None,
            timestamp,
            crc_num: calc_crc32(&data)
        }
    }

    pub fn decode(r: Record) -> kp::Record {
        let key = r.key.map(Bytes::from);
        let offset = r.offset.map(|o| o as i64).unwrap_or(-1);
        let timestamp = match r.timestamp {
            0 => kp::NO_TIMESTAMP,
            t => t as i64,
        };

        let mut headers = IndexMap::new();
        if let Some(hs) = r.header {
            for Header { name, value } in hs {
                headers.insert(StrBytes::from(name), Some(Bytes::from(value)));
            }
        }
        let value = if r.data.is_empty() {
            None
        } else {
            Some(Bytes::from(r.data))
        };

        kp::Record {
            transactional: false,
            control: false,
            partition_leader_epoch: kp::NO_PARTITION_LEADER_EPOCH,
            producer_id: kp::NO_PRODUCER_ID,
            producer_epoch: kp::NO_PRODUCER_EPOCH,
            timestamp_type: kp::TimestampType::Creation,
            offset,
            sequence: kp::NO_SEQUENCE,
            timestamp,
            key,
            value,
            headers,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use kafka_protocol::records::Record as KpRecord;

    #[test]
    fn test_encode_decode() {
        let original_record = KpRecord {
            transactional: false,
            control: false,
            partition_leader_epoch: kp::NO_PARTITION_LEADER_EPOCH,
            producer_id: kp::NO_PRODUCER_ID,
            producer_epoch: kp::NO_PRODUCER_EPOCH,
            timestamp_type: kp::TimestampType::Creation,
            offset: 42,
            sequence: kp::NO_SEQUENCE,
            timestamp: 1625079600000,
            key: Some(Bytes::from("test_key")),
            value: Some(Bytes::from("test_value")),
            headers: {
                let mut headers = IndexMap::new();
                headers.insert(StrBytes::from("header1"), Some(Bytes::from("value1")));
                headers.insert(StrBytes::from("header2"), Some(Bytes::from("value2")));
                headers
            },
        };

        let record = Message::encode(original_record.clone());
        let decoded_record = Message::decode(record);

        assert_eq!(original_record.offset, decoded_record.offset);
        assert_eq!(original_record.timestamp, decoded_record.timestamp);
        assert_eq!(original_record.key, decoded_record.key);
        assert_eq!(original_record.value, decoded_record.value);
        assert_eq!(original_record.headers, decoded_record.headers);
    }
}