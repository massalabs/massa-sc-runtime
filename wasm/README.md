# Folder webassembly

## Tests binaries

Code dumped from AssemblyScript with the command line
```
npm run asbuild:untouched -- --exportRuntime
```

- get_string.wat
```ts
export function getString(): string {
  return "hello test";
}
```

- caller.wat
```ts
export declare function call(address: string, func:string): string
export declare function print(message: string): void

export function main(): i32 {
  let string_from = call("get_string.wat", "getString")
  print(string_from)
  return 0;
}
```
