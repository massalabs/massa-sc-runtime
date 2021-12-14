"use strict";
exports.__esModule = true;
var fs_1 = require("fs");
var loader_1 = require("@assemblyscript/loader"); // or require
(0, loader_1.instantiate)(
// Binary to instantiate
(0, fs_1.readFileSync)("build/untouched.wasm"),
// or fetch, or fs.promises.readFile, or just a buffer @adrien-zinger
// Additional imports
{ /*...*/}).then(function (_a) {
    var exports = _a.exports;
    /*...*/
});
