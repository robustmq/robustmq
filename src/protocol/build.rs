extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["src/robust/eraftpb.proto"],
                                &["src/robust"]).unwrap();
}
