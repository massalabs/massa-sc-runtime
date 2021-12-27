import { print, call } from "./massa";
import { Animal } from "./animal_lib";
import { JSON } from "json-as";

export function main() {
    let animal = JSON.parse<Animal>(call("animal_sc", "get_animal", "dog"));
    print(`I have just created a ${animal.name}`);
    
}
