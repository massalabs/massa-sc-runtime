import { Animal } from "./animal_lib";
import { JSON } from "json-as";
import { print, call } from "./massa";

export function main() {
    let animals: Array<Animal> = [
        { name: "wolf", sound: "wof wof" },
        { name: "bird", sound: "tweet tweet" }
    ];
    let addr = call("animal_sc_address", "new_zoo", JSON.stringify(animals));
    print(`My new zoo is here: ${addr}`);
}