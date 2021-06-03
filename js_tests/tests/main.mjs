export async function run_tests(ffi) {
    var assert;
    try {
        assert = (await import('assert')).strict;
    } catch(__) {}

    if (!assert) {
        assert = {
            equal: function (lhs, rhs) {
                if (lhs !== rhs) {
                    throw new Error(`Assertion failed: '${lhs} == ${rhs}'`);
                }
            },
            deepEqual: function (x, y) {
                if (x === y) {
                  return true;
                }
                else if ((typeof x == "object" && x != null) && (typeof y == "object" && y != null)) {
                  if (Object.keys(x).length != Object.keys(y).length)
                    return false;

                  for (var prop in x) {
                    if (y.hasOwnProperty(prop))
                    {
                      if (! this.deepEqual(x[prop], y[prop]))
                        return false;
                    }
                    else
                      return false;
                  }

                  return true;
                }
                else
                  return false;
            },
        };
    }

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
            Uint8Array.from('Hello, \0', c => c.charCodeAt(0)),
            Uint8Array.from('null termination!\0', c => c.charCodeAt(0)),
        )),
        'Hello, null termination!',
    );

    assert.deepEqual(
        ffi.boxCBytesIntoBuffer(ffi.concat_bytes(
            Uint8Array.from('Hello, ', c => c.charCodeAt(0)),
            Uint8Array.from('World!', c => c.charCodeAt(0)),
        )),
        Uint8Array.from('Hello, World!', c => c.charCodeAt(0)),
    );

    function wrap_cb_for_ffi(f) {
        return (send_ret, ...args) => {
            try {
                return send_ret(f(...args));
            } catch (e) {
                console.error(e);
            }
        };
    }

    assert.deepEqual(
        ffi.call_with_42(wrap_cb_for_ffi((n) => {
            assert.deepEqual(n, 42);
            console.log(n); // 42
            return 27;
        })),
        27,
    );

    assert.deepEqual(
        ffi.call_with_42(wrap_cb_for_ffi((n) => {
            assert.deepEqual(n, 42);
            console.log(n); // 42
            return 27;
        })),
        27,
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

    assertCheckPointIsCalled((checkPoint) => {
        ffi.call_with_str(wrap_cb_for_ffi((s) => {
            s = ffi.refCStringToString(s);
            assert.deepEqual(s, "Hello, World!");
            checkPoint();
        }));
    });

    assertCheckPointIsCalled((checkPoint) => {
        let error = null;
        let v = ffi.withOutVecOfPtrs("Vec_uint8_t", "uint8_t", (p) => {
            try {
                checkPoint();
                ffi.takes_out_vec(p);
            } catch(e) {
                error = e;
            }
        });
        if (error) { throw error; }
        console.log(v);
    });

    assertCheckPointIsCalled((checkPoint) => {
        let error = null;
        let v = ffi.withOutBoxCBytes((p) => {
            try {
                checkPoint();
                ffi.takes_out_slice(p);
            } catch(e) {
                error = e;
            }
        });
        if (error) { throw error; }
        console.log(v);
    });

    assert.deepEqual(
        [
            ffi.boolify("True"),
            ffi.boolify("False"),
        ],
        [true, false],
    );

    assert.deepEqual(
        [
            ffi.boolify2("True"),
            ffi.boolify2("False"),
        ],
        [true, false],
    );

    (async function() {
        const start = performance.now();
        const ffi_long_running = ffi.long_running();
        const end = performance.now();
        const duration = end - start;
        assert(duration < 2.0); // Not more than 2 ms to perform the call.
        assert.deepEqual(
            await Promise.race(
                [
                    ffi_long_running.then(() => "long_running"),
                    new Promise((resolve, reject) => {
                        setTimeout(resolve, 10, "short_running");
                    }),
                ]
            ),
            "short_running",
        );
        assert.deepEqual(await ffi_long_running, 42);
    })()

    console.log('Js tests passed successfully ✅');
}
