// Copyright (c) 2023 MASSA LABS <info@massa.net>
//
//! ## **Overview**
//!
//! This module contains Protobuf message definitions for the Massa blockchain ABI.
//! It uses utilizes the `prost-build` tool to generate Rust code from the Protobuf definitions.
//!
//! ## **Structure**
//!
//! * `build.rs`: This file contains build instructions for generating Rust code from the Protobuf definitions using the `prost-build` tool.
//! * `proto/`: This directory contains the Protobuf message definitions for the Massa blockchain ABI
//! * `src/wasmv1_execution/abi/proto`: This directory contains the generated Rust code for the Protobuf message definitions.
//! It also includes a `_includes.rs` file for importing the generated Rust modules and an `abi.bin` file for server reflection protocol.
//!
//! ## **Usage**
//! To use this module, simply include it as a dependency in your Rust project's `Cargo.toml` file.
//! You can then import the necessary Rust modules for the Massa ABI and use the Protobuf messages as needed.
//!

/// Massa protos Module
pub mod massa {
    /// Massa ABI Module
    pub mod abi {
        /// Version 1 of the Massa protos
        pub mod v1 {
            include!("massa.abi.v1.rs");
        }
    }
}
