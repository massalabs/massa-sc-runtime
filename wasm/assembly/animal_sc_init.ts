import { set_for, call, include_bytes, create_sc } from "./massa";
import { JSON } from "json-as";
import { Animal } from "./animal_lib";

export function main(_arg: string) {
    // Replaced in compile time
    let bytecode = include_bytes("./zoo.wasm");
    let addr = create_sc(bytecode).address;
    for (let i = 0; i < 50; ++i) {
        let animal = JSON.parse<Animal>(call("animal_randomizer", "generate", ""));
        set_for(addr, animal.name, animal.sound);
    }
    return 0;
}
