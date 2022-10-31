import { print } from "massa-sc-std";

export function main(): void {
    let rnd = seed();
    let d = Date.now();
    let a = new Date(d);
    print(`${a}`);
    abort("abort main", "use_builtins.ts");
}

export function abort_1(_: string): void {
    abort("abort 1");
}

export function abort_2(_: string): void {
    abort("abort 2", "blop", 2);
}

export function abort_3(_: string): void {
    abort("abort 3", "blop", 2, 3);
}

export function use_trace_1(_: string): void {
    trace("hello world");
}

export function use_trace_2(_: string): void {
    trace("hello world", 5, .1, .2, .3, .4, .5);
}
