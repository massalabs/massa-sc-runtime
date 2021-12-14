export declare function call(address: string, func:string): string
export declare function print(message: string): void

export function main(): i32 {
  let string_from = call("get_string.wat", "getString")
  print(string_from)
  return 0;
}
