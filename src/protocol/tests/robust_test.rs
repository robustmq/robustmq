#[cfg(test)]
mod tests {
    #[test]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/robust") // you can change the generated code's location
            .compile(
                &["src/robust/broker.proto"],
                &["src/robust/"], // specify the root location to search proto dependencies
            )
            .unwrap();
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/robust") // you can change the generated code's location
            .compile(
                &["src/robust/meta.proto"],
                &["src/robust/"], // specify the root location to search proto dependencies
            )
            .unwrap();
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/robust") // you can change the generated code's location
            .compile(
                &["src/robust/eraftpb.proto"],
                &["src/robust/"], // specify the root location to search proto dependencies
            )
            .unwrap();
    }
}
