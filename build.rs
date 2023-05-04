// Copyright (c) 2023 MASSA LABS <info@massa.net>

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "prost-build")]
    tonic::build()?;

    Ok(())
}

#[cfg(feature = "prost-build")]
mod tonic {
    use glob::glob;
    use std::{path::PathBuf, process::Command};

    /// This function is responsible for building the Massa protobuf abi and generating documentation
    pub fn build() -> Result<(), Box<dyn std::error::Error>> {
        // Find all protobuf files in the 'proto/massa/' directory
        let protos = find_protos("proto/massa/")?;

        // Configure and compile the protobuf ABI
        prost_build::Config::new()
            .include_file("_includes.rs")
            .out_dir("src/wasmv1_execution/abi/proto")
            .compile_protos(&protos, &["proto/massa/abi/v1/"])
            .map_err(|e| format!("protobuf compilation error: {:?}", e))?;

        // Generate documentation for the protobuf abi
        generate_doc(&protos).map_err(|e| format!("protobuf documentation error: {:?}", e))?;

        // Return Ok if the build and documentation generation were successful
        Ok(())
    }

    /// Find all .proto files in the specified directory and its subdirectories
    fn find_protos(dir_path: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let glob_pattern = format!("{dir_path}/**/*.proto", dir_path = dir_path);
        let paths = glob(&glob_pattern)?.flatten().collect();

        Ok(paths)
    }

    /// Generate markdown and HTML documentation for the given protocol buffer files
    fn generate_doc(protos: &Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        // Generate markdown documentation using protoc.
        let protoc_md_cmd_output = Command::new("protoc")
            .args(protos)
            .arg("--proto_path=./proto/massa/abi/v1")
            .arg("--doc_out=./doc")
            .arg("--doc_opt=markdown,abi.md")
            .output()?;

        // If protoc failed to generate markdown documentation, return an error.
        if !protoc_md_cmd_output.status.success() {
            return Err(format!(
                "protoc generate MARKDOWN documentation failed: {}",
                String::from_utf8_lossy(&protoc_md_cmd_output.stderr)
            )
            .into());
        }

        // Generate HTML documentation using protoc.
        let protoc_html_cmd_output = Command::new("protoc")
            .args(protos)
            .arg("--proto_path=./proto/massa/abi/v1")
            .arg("--doc_out=./doc")
            .arg("--doc_opt=html,index.html")
            .output()?;

        // If protoc failed to generate HTML documentation, return an error.
        if !protoc_html_cmd_output.status.success() {
            return Err(format!(
                "protoc generate HTML documentation failed: {}",
                String::from_utf8_lossy(&protoc_md_cmd_output.stderr)
            )
            .into());
        }

        Ok(())
    }
}
