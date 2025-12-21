// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;
    Ok(())
}

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{:?}", proto_root);

    // Declare dependencies for all proto files and directories
    println!(
        "cargo:rerun-if-changed={}",
        proto_root.join("src/broker/*.proto").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        proto_root.join("src/meta/*.proto").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        proto_root.join("src/*.proto").display()
    );


    // Broker
    tonic_build::configure().build_server(true).compile_protos(
        &[
            proto_root.join("src/broker/mqtt.proto").to_str().unwrap(),
            proto_root
                .join("src/broker/storage.proto")
                .to_str()
                .unwrap(),
            proto_root.join("src/broker/common.proto").to_str().unwrap(),
        ],
        &[proto_root.join("src/").to_str().unwrap()],
    )?;

    // meta service
    let config = {
        let mut c = prost_build::Config::new();
        c.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
        c.protoc_arg("--experimental_allow_proto3_optional");
        c.service_generator(tonic_build::configure().service_generator());
        c
    };
    prost_validate_build::Builder::new().compile_protos_with_config(
        config,
        &[
            proto_root.join("src/meta/journal.proto").to_str().unwrap(),
            proto_root.join("src/meta/mqtt.proto").to_str().unwrap(),
            proto_root.join("src/meta/common.proto").to_str().unwrap(),
            proto_root.join("src/meta/validate.proto").to_str().unwrap(),
        ],
        &[proto_root.join("src/").to_str().unwrap()],
    )?;
    Ok(())
}
