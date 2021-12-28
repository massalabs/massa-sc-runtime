export declare function assembly_script_call(address: string, func: string, param: string): string
export declare function assembly_script_print(message: string): void
export declare function assembly_script_create_sc(bytecode: ArrayBuffer): string
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

export function create_sc(bytecode: ArrayBuffer): string {
    return assembly_script_create_sc(bytecode);
}

export function include_arr(path: string): Array<u8> {
    /* NOT IMPLEMENTED HERE */
    abort('Please use massa tool compilation')
}
