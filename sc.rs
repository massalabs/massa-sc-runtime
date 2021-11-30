// rustc --target wasm32-wasi sc.rs

fn main() {
    assert_eq!(std::env::var("KEY").unwrap(), "Value");
    println!("Hello, {}", std::env::args().nth(1).unwrap());
}
