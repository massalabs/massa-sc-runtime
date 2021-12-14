// The entry file of your WebAssembly module.

export declare function concat_world(arg: string): string;

export function main(): i32 {
 let hello = concat_world("hello");
 concat_world(hello);
 return hello.length;
}