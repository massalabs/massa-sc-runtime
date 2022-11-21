import { getOpKeys, hasOpKey, getOpData, print } from "@massalabs/massa-as-sdk";
// import { generateEvent } from "@massalabs/massa-as-sdk";

export function main(_args: string): void {

    let keys: Array<StaticArray<u8>> = getOpKeys();
    // generateEvent(`keys len: ${keys.length}`);
    if (keys.length != 3) {
        abort!(`Expect keys length to be == 2 ano not: ${keys.length}`);
    }

    for(let i=0; i<keys.length; i++) {
        let k = keys[i];
        let has_key = hasOpKey(k);
        if (!has_key) {
            abort(`Expect key: ${k} to be in op keys: ${keys}`);
        }
        if (has_key) {
            // generateEvent(`get data for ${k}`);
            let data = getOpData(k);
            if (data.length != 1 && data.length != 2) {
                abort("Expect data length to be either 1 or 2");
            }
            let msg = `data: ${data}`;
            print(msg);
            // generateEvent(msg);
        }
    }
}