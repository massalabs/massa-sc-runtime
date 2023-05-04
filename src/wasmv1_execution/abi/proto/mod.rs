pub use prost::Message;

// Include the `abi` module, which is generated from the proto files.
include!(concat!(env!("OUT_DIR"), "/abi.rs"));
