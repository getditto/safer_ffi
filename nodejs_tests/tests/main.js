const ffi = require('.');
const assert = require('assert').strict;

assert.equal(
    ffi.add(42, 27),
    42 + 27,
);

assert.equal(
    ffi.sub(2, 5),
    256 - 3,
);

assert.equal(
    ffi.get_hello(),
    'Hello, World!',
);

console.log('Node.js FFI tests passed successfully ✅');
