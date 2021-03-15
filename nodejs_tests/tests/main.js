const ffi = require('.');
const assert = require('assert').strict;

function assertCheckPointIsCalled(cb) {
    var called = false;
    cb(() => called = true);
    assert.equal(called, true);
}

// Tests:

assert.equal(
    ffi.add(42, 27),
    42 + 27,
);

(() => {
    let foo = ffi.foo_new();

    assert.equal(ffi.foo_read(foo), 42);

    ffi.foo_free(foo);
})();

assert.equal(
    ffi.boxCStringIntoString(ffi.get_hello()),
    'Hello, World!',
);

assertCheckPointIsCalled((checkPoint) => {
    ffi.withCString("Hello, World!", (s) => {
        ffi.print(s);
        checkPoint();
    });
});

assertCheckPointIsCalled((checkPoint) => {
    ffi.withCString("Hello, ", (s1) => {
        ffi.withCString("World!", (s2) => {
            checkPoint();
            let s = ffi.boxCStringIntoString(ffi.concat(s1, s2));
            assert.equal(s, "Hello, World!");
        });
    });
});

assert.equal(
    ffi.boxCStringIntoString(ffi.concat(
        Buffer.from('Hello, \0'),
        Buffer.from('null termination!\0'),
    )),
    'Hello, null termination!',
);

assert.deepEqual(
    ffi.boxCBytesIntoBuffer(ffi.concat_bytes(
        Buffer.from('Hello, '),
        Buffer.from('World!'),
    )),
    Buffer.from('Hello, World!'),
);

assertCheckPointIsCalled((checkPoint) => {
    assert.deepEqual(
        ffi.withOutBoolean((out_b) => {
            ffi.set_bool(out_b);
            checkPoint();
        }),
        true,
    );
})

console.log('Node.js FFI tests passed successfully ✅');
