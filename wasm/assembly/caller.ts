import { print, call } from "./massa";

export function main(_args: string): i32 {
  let string_from = call("get_string", "helloName", "you");
  print(string_from);
  return 0;
}
