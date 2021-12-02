use execution::run;

#[test]
fn test_caller() {
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/get_string.wat"
    ));
    run("get_string.wat".to_string(), module).expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/caller.wat"
    ));
    run("caller.wat".to_string(), module).expect("Failed to run caller.wat");
}