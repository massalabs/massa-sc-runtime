import { JSON } from "json-as";
import { Animal } from "./animal_lib";
import { print, get, set } from "./massa";

export function get_animal(name: string): string {
  // the getter will look at the current smart contract DB
  // if there is a key "name"
  return JSON.stringify<Animal>({name, sound: get(name)});
}

export function add_animal(_animal: string): string {
  // You can limit who can ad using the get_context() ABI
  let animal = JSON.parse<Animal>(_animal);
  set(animal.name, animal.sound);
  return `Animal added (${animal.sound})`;
}

// Makes the animal speak
export function speak(_animal: string): string {
  let animal = JSON.parse<Animal>(_animal);
  print(`${animal.name} says: ${animal.sound}!`);
  return "0";
}
