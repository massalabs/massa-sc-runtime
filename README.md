# Massa Smart Contracts Runtime

This is a VM (basically a wrapper around [Wasmer](https://wasmer.io/)) that lets run WASM SC generated with AssemblyScript (and using Massa customs ABIs).

## Protoc dependency

This section is mostly a copy/paste from [https://github.com/massalabs/massa/blob/main/massa-grpc/README.md](https://github.com/massalabs/massa/blob/main/massa-grpc/README.md).

As this project evolves specificities will be included.

In order to compile proto files, you must have the `protoc` compiler installed on your system. `protoc` is a protocol buffer compiler that can generate code in a variety of programming languages.

To check if you have `protoc` installed on your system, you can run the following command in your terminal:

```
protoc --version
```

If you see a version number printed out, then you have `protoc` installed. If not, you will need to download and install it.

Installing Protoc
-----------------

### macOS

To install `protoc` on macOS using Homebrew, run the following command:

```
brew install protobuf
protoc --version  # Ensure compiler version is 3+
```

### Linux

To install `protoc` on Linux, you can download the binary file for your architecture from the [official Protobuf releases page](https://github.com/protocolbuffers/protobuf/releases). Once downloaded, extract the contents of the archive and move the `protoc` binary to a location on your system PATH.

Alternatively, you can use your distribution's package manager to install `protoc`. On Ubuntu, for example, you can run:

```
sudo apt install protobuf-compiler
protoc --version  # Ensure compiler version is 3+
```

### Windows

To install `protoc` on Windows, you can download the binary file for your architecture from the [official Protobuf releases page](https://github.com/protocolbuffers/protobuf/releases). Once downloaded, extract the contents of the archive and move the `protoc` binary to a location on your system PATH.

After installing `protoc`, you should be able to compile proto files using the appropriate language-specific plugin (e.g. `protoc --go_out=./ path/to/my_proto_file.proto`).


After installing `protoc`, please verify that the `protoc` command is accessible by running `protoc --version` again and ensure compiler version is 3+.


To keep the documentation synchronised with our proto files, you must install `protoc-gen-doc`. You can use your package manager or download the binary from the official [GitHub repository releases](https://github.com/pseudomuto/protoc-gen-doc/releases) and add it to your system's `PATH`


Project build and run
---------------------

The project is set up to automatically compile proto files during the build process using
[massa-sc-runtime/build.rs](.//build.rs).
