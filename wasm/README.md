## How to build WASM test binaries

Code dumped from AssemblyScript with the command line

```shell
yarn build
```

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
}
```
