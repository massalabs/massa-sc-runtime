<!-- Copyright (c) 2023 MASSA LABS <info@massa.net> -->

### Massa ABI Protobuf

In order to compile proto files, you must have the `protoc` compiler installed on your system. `protoc` is a protocol buffer compiler that can generate code in a variety of programming languages.

To check if you have `protoc` installed on your system, you can run the following command in your terminal:

```
protoc --version
libprotoc 3.21.12 # Ensure compiler version is 3.15+
```

If you see a version number printed out, then you have `protoc` installed. If not, you will need to download and install it.

Installing Protoc
-----------------

### macOS

To install `protoc` on macOS using Homebrew, run the following command:

```
brew install protobuf
protoc --version  # Ensure compiler version is 3.15+
```

### Linux

To install `protoc` on Linux, you can download the binary file for your architecture from the [official Protobuf releases page](https://github.com/protocolbuffers/protobuf/releases). Once downloaded, extract the contents of the archive and move the `protoc` binary to a location on your system PATH.

Alternatively, you can use your distribution's package manager to install `protoc`. On Ubuntu, for example, you can run:

```
sudo apt install protobuf-compiler
protoc --version  # Ensure compiler version is 3.15+
```

### Windows

To install `protoc` on Windows, you can download the binary file for your architecture from the [official Protobuf releases page](https://github.com/protocolbuffers/protobuf/releases). Once downloaded, extract the contents of the archive and move the `protoc` binary to a location on your system PATH.

After installing `protoc`, you should be able to compile proto files using the appropriate language-specific plugin (e.g. `protoc --go_out=./ path/to/my_proto_file.proto`).

After installing `protoc`, please verify that the `protoc` command is accessible by running `protoc --version` again and ensure compiler version is 3+.

To keep the documentation synchronised with our proto files, you must install `protoc-gen-doc`. You can use your package manager or download the binary from the official [GitHub repository releases](https://github.com/pseudomuto/protoc-gen-doc/releases) and add it to your system's `PATH`


Project build and run
---------------------

The project is set up to automatically compile proto files during the build process using 
[build.rs](../build.rs).

When the project is built, `build.rs` is executed and it uses the `prost-build` crate to generate Rust code from the proto files. The generated Rust code could be found in [proto/](../src/wasmv1_execution/abi/proto).

By default, `prost-build` feature is disabled, you can update the generated code and documentation from protobuf files by running: 
```bash
cargo build --features prost-build
```

VSCode integration
------------------

1- Install [vscode-proto3](https://marketplace.visualstudio.com/items?itemName=zxh404.vscode-proto3) extension.

2- The following settings contain a `protoc` configuration block:

```json
{
    // "rust-analyzer.rust.features": ["prost-build"], // Enables the prost-build feature for the Rust Analyzer extension.
    "rust-analyzer.procMacro.enable": true,  // Enables Rust macro support for the Rust Analyzer extension.
    "rust-analyzer.cargo.buildScripts.enable": true,  // Enables cargo build scripts for the Rust Analyzer extension.
    "protoc": {  // Specifies the configuration for the protoc plugin.
        "path": "/path/to/protoc",  // Sets the path to the protoc binary that will be used to compile the protobuf files.
        "compile_on_save": true,  // Enables automatic compilation of protobuf files when they are saved.
        "options": [  // Specifies the command line options that will be passed to protoc.
            "{workspaceRoot}/proto/**/*.proto",  // Specifies the path to the protobuf files that should be compiled.
            "--proto_path=${workspaceRoot}/proto/massa/abi/v1",  // Specifies the directory to search for imported protobuf files.third-party protobuf files.
            // "--java_out=${workspaceRoot}/target/",  // Generates Java code from the protobuf files.
            // "--doc_out=${workspaceRoot}/doc/",  // Generates documentation in HTML/markdown format from the protobuf files.
            // "--doc_opt=html,index.html",  // Specifies the options for generating the HTML documentation.
            // "--doc_opt=markdown,docs.md",  // Specifies the options for generating the markdown documentation.
            // "--descriptor_set_out=${workspaceRoot}/src/abi.bin"  // Generates a binary descriptor set for the protobuf files which is used for server reflection.
        ]
    }
}

```

3- Add the snippet above to `.vscode/settings.json`.


Protoc examples
---------------

Generate html documentation:
```bash
protoc \
  ./proto/massa/**/*.proto \
  --proto_path=./proto/massa/abi/v1 \
  --doc_out=./doc/ \
  --doc_opt=html,index.html
```

Generate markdown documentation:
```bash
protoc \
  ./proto/massa/**/*.proto \
  --proto_path=./proto/massa/abi/v1 \
  --doc_out=./doc/ \
  --doc_opt=markdown,abi.md
```

Test code generation:
```bash
protoc \
  ./proto/**/*.proto \
  --proto_path=./proto/massa/abi/v1 \
  --java_out=./target/
```