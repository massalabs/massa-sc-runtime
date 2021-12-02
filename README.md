# Smart contract engine example

This is how we can see the basics of a smart contract with AssemblyScripts.

```
cargo run -- first_contract.wasm second_contract.wasm
```

## WASM files

Generated files to test this projects documented [here](wasm/README.md)

## Roadmap
- add the metering concepts
- add ABIs to get the current context (address, callstack and caller)
- abstract the ledger by defining traits.
