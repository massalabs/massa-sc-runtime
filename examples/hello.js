// This could be hide into `massa` module
import { getenv } from "std"
const args = JSON.parse(getenv("_args"));

console.log(`Hello, ${args.name}`)
// Now you,re ready to write a JS smart contract