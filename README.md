# Smart contract engine example

[![CI](https://github.com/massalabs/sc-runtime/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/massalabs/sc-runtime/actions/workflows/ci.yml?query=branch%3Amain)
[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/40217)
[![Coverage Status](https://coveralls.io/repos/github/massalabs/sc-runtime/badge.svg?branch=main)](https://coveralls.io/github/massalabs/sc-runtime?branch=main)
[![Docs](https://img.shields.io/static/v1?label=Docs&message=massalabs.github.io&color=blue)](https://massalabs.github.io/sc-runtime/assembly_simulator/)

This is how we can see the basics of a smart contract with AssemblyScripts.

```
cargo run -- first_contract.wasm second_contract.wasm
```

## WASM files

Generated files to test this projects documented [here](wasm/README.md).

## Roadmap
- [ ] add the metering concepts
- [ ] add ABIs to get the current context (address, callstack and caller)
- [ ] abstract the ledger by defining traits
