import { print, call } from "massa-sc-std";

export function main(): void {
  let string_from = call("get_string", "helloName", "you", 0);
  print(string_from);
}
