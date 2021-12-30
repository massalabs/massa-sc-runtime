export declare function assembly_script_call(address: string, func: string, param: string): string
export declare function assembly_script_print(message: string): void
/**
 * called by create_sc
 **/
export declare function assembly_script_create_sc(bytecode: string): string
export declare function get_remaining_points(): u64

export function call(address: string, func: string, param: string): string {
    return assembly_script_call(address, func, param);
}

export function print(message: string): void {
    assembly_script_print(message);
}

export function get_remaining_gas(): u64 {
    return get_remaining_points();
}

/**
 * Take a base64 string representing the module binary and create an entry in
 * the ledger.
 * 
 * The context allow you to write in this smart contract while you're executing
 * the current bytecode.
 * 
 * @param bytecode string base64 of the ledger
 * @returns Created entry address
 */
export function create_sc(bytecode: string): string {
    return assembly_script_create_sc(bytecode);
}

/**
 * Empty function that can be replaced before the compilation with
 * include_base64.js.
 * 
 * ```bash
 * node massa_tools/include_base64.js assembly/create_sc.ts && asc assembly/create_sc.m.ts --target release --exportRuntime
 * ```
 * @param _path 
 */
export function include_base64(_path: string): string {
    /* NOT IMPLEMENTED HERE */
    abort('Please use massa tool *include_base64* compilation')
}
