export declare function assembly_script_call(address: string, func: string, param: string): string
export declare function assembly_script_print(message: string): void
export declare function assembly_script_set(key: string, value: string): void
export declare function assembly_script_get(key: string): string
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

export function get(key: string): string {
    return "";
}

export function set(key: string, value: string): void {

}

export function create_sc(bytecode: string): SmartContract {
    return { address: "", bytecode };
}

export function get_bytecode(address: string): string {
    return "";
}

export function self(): string {
    return "";
}

export function include_bytes(path: string): string {
    return "";
}
export class SmartContract {
    address: string
    bytecode: string
}