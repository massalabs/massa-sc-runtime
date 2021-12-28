const fs = require('fs');
const args = process.argv.slice(2);
const data = fs.readFileSync(args[0], 'base64');
console.log(Buffer.from(data).join(','));