const fs = require('fs');
const file = process.argv[2];
const include_bytes_regex = /include_arr\(["']+([\.a-z_\-\/\\]*)["']+\)[;]+/i;
const lines = String(fs.readFileSync(file)).split('\n').map(line => {
    let res = line.match(include_bytes_regex);
    if (res != null) {
        const data = fs.readFileSync(res[1], 'binary');
        const arrByte = Array.from(Buffer.from(data));
        line = line.replace(res[0], JSON.stringify(arrByte));
    }
    return line;
});
fs.writeFileSync(file, lines.join('\n'), { flag: 'w' });
