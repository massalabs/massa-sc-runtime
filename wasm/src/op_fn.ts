import { getOpKeys, hasOpKey, getOpData, print } from "@massalabs/massa-as-sdk";

export function main(_args: string): void {
    let keys: Array<Uint8Array> = getOpKeys();
    for(let i=0; i<keys.length; i++) {
        let k = keys[i];
        if (hasOpKey(k)) {
            let data = getOpData(k);
            print(`data: ${data}`);
        }
    }
}
