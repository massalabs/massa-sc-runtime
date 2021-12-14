import { readFileSync } from "fs"
import { instantiate } from "@assemblyscript/loader"; // or require
instantiate(
  // Binary to instantiate
  readFileSync("build/untouched.wasm"),
  // or fetch, or fs.promises.readFile, or just a buffer @adrien-zinger
  // Additional imports
  { /*...*/ }
).then(({ exports }) => {
  /*...*/
})
