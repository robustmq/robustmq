#[cfg(test)]
mod tests {
    #[test]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/broker_server/generate") // you can change the generated code's location
            .compile(
                &[
                    "src/broker_server/proto/mqtt.proto",
                    ],
                &["src/broker_server/proto"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(true)
            .out_dir("src/placement_center/generate") // you can change the generated code's location
            .compile(
                &[
                    "src/placement_center/proto/common.proto",
                    "src/placement_center/proto/engine.proto",
                    "src/placement_center/proto/kv.proto",
                    "src/placement_center/proto/placement.proto",
                    ],
                &["src/placement_center/proto"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(false)
            .out_dir("src/storage_engine/generate/protocol") // you can change the generated code's location
            .compile(
                &[
                    "src/storage_engine/proto/protocol/header.proto",
                    "src/storage_engine/proto/protocol/fetch.proto",
                    "src/storage_engine/proto/protocol/produce.proto",
                ],
                &["src/storage_engine/proto/protocol/"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(false)
            .out_dir("src/storage_engine/generate/record") // you can change the generated code's location
            .compile(
                &[
                    "src/storage_engine/proto/record/record.proto",
                ],
                &["src/storage_engine/proto/record"], // specify the root location to search proto dependencies
            )
            .unwrap();
    }
}
