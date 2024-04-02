use crate::{metadata::cache::MetadataCache, server::MQTTProtocol};
use protocol::mqtt::{ConnAck, ConnectReturnCode, MQTTPacket};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MQTTAckBuild {
    protocol: MQTTProtocol,
    metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl MQTTAckBuild {
    pub fn new(protocol: MQTTProtocol, metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return MQTTAckBuild {
            protocol,
            metadata_cache,
        };
    }
    pub fn conn_ack(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn pub_ack(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn pub_rec(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn pub_rel(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn pub_comp(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn ping_resp(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn sub_ack(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn unsub_ack(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }

    pub fn distinct(&self) -> MQTTPacket {
        let conn_ack = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(conn_ack, None);
    }
}
