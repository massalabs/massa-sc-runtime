import { print, call } from "./massa";

export function main(): i32 {
  let string_from = call("get_string.wat", "getString")
  print(string_from)
  return 0;
}