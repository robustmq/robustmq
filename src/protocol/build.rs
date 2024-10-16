fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/broker_server/proto/admin.proto")?;
    tonic_build::compile_protos("src/broker_server/proto/placement.proto")?;

    tonic_build::compile_protos("src/journal_server/proto/protocol/admin.proto")?;
    tonic_build::compile_protos("src/journal_server/proto/protocol/engine.proto")?;
    tonic_build::compile_protos("src/journal_server/proto/protocol/inner.proto")?;
    tonic_build::compile_protos("src/journal_server/proto/record/record.proto")?;

    tonic_build::compile_protos("src/placement_center/proto/common.proto")?;
    tonic_build::compile_protos("src/placement_center/proto/journal.proto")?;
    tonic_build::compile_protos("src/placement_center/proto/kv.proto")?;
    tonic_build::compile_protos("src/placement_center/proto/mqtt.proto")?;
    tonic_build::compile_protos("src/placement_center/proto/openraft.proto")?;
    tonic_build::compile_protos("src/placement_center/proto/placement.proto")?;
    Ok(())
}