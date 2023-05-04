extern crate prost_build;

fn main() {
    prost_build::compile_protos(
        &["src/wasmv1_execution/abi/proto/abi.proto"],
        &["src/wasmv1_execution/abi/proto"],
    )
    .unwrap();
}
