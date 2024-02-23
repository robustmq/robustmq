#[cfg(test)]
mod tests {
    #[test]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/broker_server") // you can change the generated code's location
            .compile(
                &["src/broker_server/broker.proto"],
                &["src/broker_server/"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(true)
            .out_dir("src/placement_center") // you can change the generated code's location
            .compile(
                &["src/placement_center/placement.proto"],
                &["src/placement_center/"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(true)
            .out_dir("src/storage_engine") // you can change the generated code's location
            .compile(
                &["src/storage_engine/storage.proto"],
                &["src/storage_engine/"], // specify the root location to search proto dependencies
            )
            .unwrap();

    }
}
