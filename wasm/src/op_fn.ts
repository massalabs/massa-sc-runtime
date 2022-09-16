import { getOpKeys, hasOpKey, getOpData } from "@massalabs/massa-sc-std";

export function main(_args: string): void {
    let keys: Array<StaticArray<u8>> = getOpKeys();
    for(let i=0; i<keys.length; i++) {
        let k = keys[i];
        if (hasOpKey(k)) {
            let data = getOpData(k);
        }
    }
}
