## How to build WASM test binaries

Code dumped from AssemblyScript with the command line

```shell
yarn build
```

- get_string.wat
```ts
export function helloName(name: string): string {
    return `hello ${name}`;
}

```

- caller.wat
```ts
import { print, call } from "./massa";

export function main(_args: string): i32 {
  let string_from = call("get_string", "helloName", "you", 0);
  print(string_from);
  return 0;
}

```
