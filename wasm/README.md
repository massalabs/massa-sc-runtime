## How to build WASM test binaries

Code dumped from AssemblyScript with the command line

```shell
yarn build
```

<<<<<<< HEAD
- get_string.wat
```ts
export function getString(name: string): string {
  let a = JSON.parse(name)
  return "hello " + a.world;
}
```

- caller.wat
```ts
export declare function call(address: string, func:string, params:string): string
export declare function print(message: string): void
export declare function how_many(): i32

export function main(): i32 {
  let a = {
    hello: "hello",
    world: "world"
  }
  let string_from = call("get_string.wat", "getString", JSON.stringify(a))
  print(string_from)
  return 0;
=======
-   `get_string.wat`

```typescript
export function getString(): string {
    return "hello test";
}
```

-   `caller.wat`

```typescript
export declare function call_module(address: string, func: string): string;
export declare function print(message: string): void;

export function main(): i32 {
    let string_from = call_module("get_string.wat", "getString");
    print(string_from);
    return 0;
>>>>>>> tmp-typed
}
```
