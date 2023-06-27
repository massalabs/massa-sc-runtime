#[cfg(feature = "build-wasm")]
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

// needed for windows compatibility
#[cfg(feature = "build-wasm")]
use which::which;

#[cfg(feature = "build-wasm")]
const GIT_REPOSITORY: &str = "https://github.com/massalabs/as_abi_protobuf.git";
#[cfg(feature = "build-wasm")]
const GIT_BRANCH: &str = "feature/Improve_ABI_types_in_wasmv1";

#[cfg(feature = "build-wasm")]
fn main() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("Failed to get Cargo dir");
    let cargo_dir = &Path::new(&manifest_dir);
    let target_path = cargo_dir.join("as_abi_protobuf");
    let target_path_exists = target_path.exists();

    if target_path_exists {
        let target_path = target_path
            .canonicalize()
            .expect("as_abi_protobuf does not exist");

        let is_symlink = target_path
            .symlink_metadata()
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false);

        if !is_symlink && target_path.join(".git").exists() {
            env::set_current_dir(&target_path).expect("cd failed");

            Command::new("git")
                .arg("pull")
                .output()
                .expect("git pull failed");
        }
    } else {
        git_clone(&target_path);
    }

    npm_install(&target_path);

    build_wasm();

    copy_wasm(target_path, cargo_dir);
}

#[cfg(feature = "build-wasm")]
fn git_clone(target_path: &PathBuf) {
    Command::new("git")
        .arg("clone")
        .arg(GIT_REPOSITORY)
        .arg(target_path.display().to_string())
        .output()
        .expect("Failed to execute git clone command");

    env::set_current_dir(target_path)
        .expect("Failed to change directory to the destination folder");

    Command::new("git")
        .arg("checkout")
        .arg(GIT_BRANCH)
        .output()
        .expect("Failed to execute git checkout command");
}

#[cfg(feature = "build-wasm")]
fn npm_install(target_path: &PathBuf) {
    env::set_current_dir(target_path)
        .expect(&format!("Failed to cd to {}", target_path.display()));

    let npm_path = which("npm").expect("npm not found in PATH");

    Command::new(npm_path)
        .arg("install")
        .output()
        .expect("npm install failed");
}

#[cfg(feature = "build-wasm")]
fn build_wasm() {
    let package_json = fs::read_to_string("package.json")
        .expect("Failed to read package.json file");
    let package_data: serde_json::Value = serde_json::from_str(&package_json)
        .expect("Failed to parse package.json file");

    let Some(scripts) = package_data["scripts"].as_object() else { return };
    let rules: Vec<String> = scripts
        .into_iter()
        .filter(|(s, _)| s.starts_with("all:"))
        .map(|(s, _)| s.clone())
        .collect();

    for rule in rules.iter() {
        let npm_path = which("npm").expect("npm not found in PATH");

        Command::new(npm_path)
            .arg("run")
            .arg(rule)
            .output()
            .expect(&format!("Failed to execute npm run {} script", rule));
    }
}

#[cfg(feature = "build-wasm")]
fn copy_wasm(target_path: PathBuf, cargo_dir: &&Path) {
    let build_dir = target_path.join("build");
    let wasm_dir = cargo_dir.join("wasm");
    if let Ok(entries) = fs::read_dir(&build_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && path.extension().unwrap_or_default() == "wasm_add"
                {
                    let file_name = path.file_name().unwrap();
                    let destination_path = wasm_dir.join(file_name);
                    fs::copy(&path, &destination_path)
                        .expect("Failed to copy the WASM file");
                }
            }
        }
    }
}

#[cfg(not(feature = "build-wasm"))]
fn main() {
    // Do nothing if the "build-wasm" feature is not defined
}
