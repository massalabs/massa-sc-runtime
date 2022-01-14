export declare function assembly_script_print(message: string): void
export declare function assembly_script_call(address: string, func: string, param: string): string
export declare function assembly_script_get_remaining_gas(): u64
export declare function assembly_script_create_sc(bytecode: string): string
export declare function assembly_script_set_data(key: string, value: string): void;
export declare function assembly_script_set_data_for(address: string, key: string, value: string): void;
export declare function assembly_script_get_data(key: string): string;
export declare function assembly_script_get_data_for(address: string, key: string): string;
export declare function assembly_script_has_data(key: string): bool;
export declare function assembly_script_has_data_for(address: string, key: string): bool;
export declare function assembly_script_get_owned_addresses(): string;
export declare function assembly_script_get_call_stack(): string;
export declare function assembly_script_generate_event(event: string): void;
export declare function assembly_script_transfer_coins(to_address: string, raw_amount: u64): void;
export declare function assembly_script_transfer_coins_for(from_address: string, to_address: string, raw_amount: u64): void;
export declare function assembly_script_get_balance(): u64;
export declare function assembly_script_get_balance_for(address: string): u64;
export declare function assembly_script_hash(data: string): string;
export declare function assembly_script_signature_verify(data: string, signature: string, public_key: string): bool;
export declare function assembly_script_address_from_public_key(public_key: string): string;
export declare function assembly_script_get_time(): u64;
export declare function assembly_script_unsafe_random(): i64;

/**
 * Prints in the node logs
 *
 * @param message Message string
 */
export function print(message: string): void {
    assembly_script_print(message);
}

/**
 * Retreive a module in the ledger at the given address and call a function
 *
 * @param address Address hash in format string
 * @param func Function name exported in the module
 * @param param String input parameters
 * @returns String output of the function called
 */
export function call(address: string, func: string, param: string): string {
    return assembly_script_call(address, func, param);
}

/**
 * Return the remaining gas available
 * @returns Gas available
 */
export function get_remaining_gas(): u64 {
    return assembly_script_get_remaining_gas();
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
 * Set data in the creator of operation ledger entry database.
 *
 * ```js
 * // Each ledger entry contains this object.
 * {
 *  sce_balance, // Amount
 *  database, // HashMap<Hash, Vec<u8>>
 *  program_data, // Vec<u8>
 * }
 * ```
 * @param key key address of the data
 * @param value value to put in the DB
 */
export function set_data(key: string, value: string): void {
    assembly_script_set_data(key, value);
}

/**
 * Set data in the creator of operation ledger entry database in a specified address. \
 * You won't be able to insert a value if you're not the direct creator of the entry \
 * or the owner of the address.
 *
 * ```js
 * // Each ledger entry contains this object.
 * {
 *  sce_balance, // Amount
 *  database, // HashMap<Hash, Vec<u8>>
 *  program_data, // Vec<u8>
 * }
 * ```
 * @param address address of a smart contract or user hash
 * @param key key address of the data
 * @param value value to put in the DB
 */
export function set_data_for(address: string, key: string, value: string): void {
    assembly_script_set_data_for(address, key, value);
}

/**
 * Get data in the creator of operation ledger entry database.
 *
 * ```js
 * // Each ledger entry contains this object.
 * {
 *  sce_balance, // Amount
 *  database, // HashMap<Hash, Vec<u8>>
 *  program_data, // Vec<u8>
 * }
 * ```
 * @param key key address of the data
 * @param value value if the key
 */
export function get_data(key: string): string {
    return assembly_script_get_data(key);
}

/**
 * Get data in the creator of operation ledger entry database in a specified address.
 *
 * ```js
 * // Each ledger entry contains this object.
 * {
 *  sce_balance, // Amount
 *  database, // HashMap<Hash, Vec<u8>>
 *  program_data, // Vec<u8>
 * }
 * ```
 * @param address address of a smart contract or user hash
 * @param key key address of the data
 * @param value value if the key
 */
export function get_data_for(address: string, key: string): string {
    return assembly_script_get_data_for(address, key);
}

/**
 * Checks whether an entry exists in the caller's datastore.
 *
 * @param key key of the data (will be hashed internally)
 * @returns true if the key was found, false otherwise
 */
export function has_data(key: string): bool {
    return assembly_script_has_data(key);
}

/**
 * Checks whether an entry exists in the datastore of an arbitrary address.
 *
 * @param address target address
 * @param key key of the data (will be hashed internally)
 * @returns true if the key was found, false otherwise
 */
export function has_data_for(address: string, key: string): bool {
    return assembly_script_has_data_for(key);
}

/**
 *  Returns an entry from the caller's datastore or a default value if not found 
 *
 * @param address target address
 * @param key key of the data (will be hashed internally)
 * @param default_value default value if not found
 * @returns found string value or default string
 */
export function get_data_or_default(key: string, default_value: string): string {
    if(has_data(key)) {
        return get_data(key);
    }
    return default_value;
}

/**
 *  Returns an entry from an address' datastore or a default value if not found 
 *
 * @param address target address
 * @param key key of the data (will be hashed internally)
 * @param default_value default value if not found
 * @returns found string value or default string
 */
export function get_data_or_default_for(address:string, key: string, default_value: string): string {
    if(has_data_for(address, key)) {
        return get_data_for(address, key);
    }
    return default_value;
}

/**
 * Get context current owned addresses.
 *
 * You can check your own address or check the addresses of the smart contract you've created during the current execution.
 *
 * @returns JSON formated list of addresses containing the owned addresses
 */
export function get_owned_addresses(): string {
    return assembly_script_get_owned_addresses();
}

/**
 * Get context current call stack
 *
 * The call stack is stack of called module. You can look all previous \
 * addresses since the address of the operation sender.
 *
 * @returns JSON formated list of addresses containing the call stack
 */
export function get_call_stack(): string {
    return assembly_script_get_call_stack();
}

/**
 * Generates an event
 *
 * @param message String version of the event
 */
export function generate_event(event: string): void {
    assembly_script_generate_event(event);
}

/**
 * Transfer SCE coins from the current address to to_address
 *
 * @param to_address Destination address hash in format string
 * @param raw_amount Raw amount (in elementary units)
 */
export function transfer_coins(to_address: string, raw_amount: u64): void {
    return assembly_script_transfer_coins(to_address, raw_amount);
}

/**
 * Transfer SCE coins from from_address to to_address
 *
 * @param from_address Source address hash in format string
 * @param to_address Destination address hash in format string
 * @param raw_amount Raw amount (in elementary units)
 */
export function transfer_coins_for(from_address: string, to_address: string, raw_amount: u64): void {
    return assembly_script_transfer_coins_for(from_address, to_address, raw_amount);
}

/**
 * Gets the balance of the current address
 *
 * @returns The raw balance of the address (in elementary nits)
 */
export function get_balance(): u64 {
    return assembly_script_get_balance();
}

/**
 * Gets the balance of the current address
 *
 * @param addres Address hash in format string
 * @returns The raw balance of the address (in elementary nits)
 */
export function get_balance_for(address: string): u64 {
    return assembly_script_get_balance_for(address);
}

/**
 * Hash data and return the base58-encoded hash
 *
 * @param data Data to hash
 */
export function hash(data: string): string {
    return assembly_script_hash(event);
}

/**
 * Hash data and return the base58-encoded hash
 *
 * @param data Data that was signed
 * @param signature base58check signature
 * @param public_key base58check public key
 * @returns true if verification suceeded, false otherwise
 */
export function signature_verify(data: string, signature: string, public_key: string): bool {
    return assembly_script_signature_verify(data, signature, public_key);
}

/**
 * Converts a public key to an address
 *
 * @param public_key Base58check public key
 * @returns the resulting address
 */
export function address_from_public_key(data: string): string {
    return assembly_script_address_from_public_key(data);
}

/**
 * Gets the slot unix timestamp in milliseconds
 *
 * @returns unix timestamp in milliseconds
 */
export function assembly_script_get_time(): u64 {
    return assembly_script_get_time();
}

/**
 * Gets an unsafe random i64 (all bits random)
 * This function is unsafe because the random draws can be predicted and manipulated by attackers.
 *
 * @returns random signed 64bit integer
 */
export function assembly_script_unsafe_random(): i64 {
    return assembly_script_unsafe_random();
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
