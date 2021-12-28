import { create_sc } from "./massa";

export function main(_args: string): i32 {
    // Create smart contract "get_string"
    const bytes = include_arr('./build/get_string.wasm');
    const arr = new Uint8Array(bytes.length);
    for (let i = 0; i < bytes.length; ++i) {
        arr[i] = bytes[i];
    }
    create_sc(arr.buffer);
    return 0;
}