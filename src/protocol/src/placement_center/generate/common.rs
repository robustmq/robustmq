#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonReply {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClusterType {
    BrokerServer = 0,
    StorageEngine = 1,
    PlacementCenter = 2,
}
impl ClusterType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ClusterType::BrokerServer => "BrokerServer",
            ClusterType::StorageEngine => "StorageEngine",
            ClusterType::PlacementCenter => "PlacementCenter",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BrokerServer" => Some(Self::BrokerServer),
            "StorageEngine" => Some(Self::StorageEngine),
            "PlacementCenter" => Some(Self::PlacementCenter),
            _ => None,
        }
    }
}
