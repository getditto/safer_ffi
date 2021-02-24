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

(() => {
    let foo = ffi.foo_new();

    assert.equal(ffi.foo_read(foo), 42);

    ffi.foo_free(foo);
})();

assert.equal(
    ffi.charPBoxedIntoString(ffi.get_hello()),
    'Hello, World!',
);

function assertCheckPointIsCalled(cb) {
    var called = false;
    cb(() => called = true);
    assert.equal(called, true);
}

assertCheckPointIsCalled((checkPoint) => {
    ffi.withFfiString("Hello, World!", (s) => {
        ffi.print(s);
        checkPoint();
    });
});

assertCheckPointIsCalled((checkPoint) => {
    ffi.withFfiString("Hello, ", (s1) => {
        ffi.withFfiString("World!", (s2) => {
            checkPoint();
            let s = ffi.charPBoxedIntoString(ffi.concat(s1, s2));
            assert.equal(s, "Hello, World!");
        });
    });
});

console.log('Node.js FFI tests passed successfully ✅');
