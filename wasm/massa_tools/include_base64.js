const fs = require('fs');
const file = process.argv[2];
const include_bytes_regex = /include_base64\(["']+([\.a-z_\-\/\\]*)["']+\)[;]+/i;
const lines = String(fs.readFileSync(file)).split('\n').map(line => {
    let res = line.match(include_bytes_regex);
    if (res != null) {
        const data = fs.readFileSync(res[1], 'base64');
        line = line.replace(res[0], JSON.stringify(data));
    }
    return line;
});
fs.writeFileSync(file.replace('.ts', '.m.ts'), lines.join('\n'), { flag: 'w+' });
