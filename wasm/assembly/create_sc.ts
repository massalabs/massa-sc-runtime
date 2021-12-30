import { create_sc, include_base64 } from "./massa";

export function main(_args: string): i32 {
    // Create smart contract "get_string"
    const bytes = include_base64('./build/get_string.wasm');
    create_sc(bytes);
    return 0;
}
