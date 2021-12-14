export declare function assembly_script_call(address: string, func:string): string
export declare function assembly_script_print(message: string): void

export function call(address: string, func: string, param: string): string {
    return assembly_script_call(address, func);
}

export function print(message: string): void {
    assembly_script_print(message);
}
