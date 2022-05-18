// Tests:
export async function run_tests({ ffi, performance, assert, is_web }) {
    ffi.setup();

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

    function assertCheckPointIsCalled(cb) {
        var called = false;
        cb(() => called = true);
        assert.equal(called, true);
    }

    assertCheckPointIsCalled((checkPoint) => {
        ffi.withCString("Hello, World!", (s) => {
            ffi.print(s);
            checkPoint();
        });
    });

    // Unsigned
    for(const bigint of [
        0, 0n,
        1, 1n,
        2 ** 53 - 2, 2n ** 53n - 2n,
        2 ** 53 - 1, 2n ** 53n - 1n,
        2n ** 53n - 0n,
        2n ** 53n + 1n,
        2n ** 53n + 2n,
        2n ** 64n - 2n,
        2n ** 64n - 1n,
    ]) {
        var bigint2;
        assertCheckPointIsCalled((checkPoint) => {
            // console.log(bigint.toString(), typeof bigint);
            var errored;
            ffi.withCString(bigint.toString(), (s) => {
                try {
                    checkPoint();
                    bigint2 = ffi.check_big_int_unsigned(
                        bigint,
                        s,
                    );
                    assert(bigint == bigint2);
                } catch(e) {
                    errored = e;
                }
            });
            if (errored) {
                throw(errored);
            }
        });
        assert(bigint == bigint2);
    }
    // Signed
    for(const bigint of [
        -(2n ** 63n),
        -(2n ** 63n - 1n),
        -(2n ** 53n + 1n),
        -(2n ** 53n - 0n),
        -(2n ** 53n - 1n), -(2 ** 53 - 1),
        -1, -1n,
        0, 0n,
        1, 1n,
        2n ** 53n - 1n, 2 ** 53 - 1,
        2n ** 53n - 0n,
        2n ** 53n + 1n,
        2n ** 63n - 1n,
    ]) {
        var bigint2;
        assertCheckPointIsCalled((checkPoint) => {
            var errored;
            ffi.withCString(bigint.toString(), (s) => {
                try {
                    checkPoint();
                    bigint2 = ffi.check_big_int_signed(
                        bigint,
                        s,
                    );
                } catch(e) {
                    errored = e;
                }
            });
            if (errored) {
                throw(errored);
            }
        });
        assert(bigint == bigint2);
    }

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
        Uint8Array.from(ffi.boxCBytesIntoBuffer(ffi.concat_byte_slices(
            Uint8Array.from('Hello, ', c => c.charCodeAt(0)),
            Uint8Array.from('World!', c => c.charCodeAt(0)),
        ))),
        Uint8Array.from('Hello, World!', c => c.charCodeAt(0)),
    );

    assert.deepEqual(
        ffi.call_with_42(wrap_cb_for_ffi((n) => {
            assert.deepEqual(n, 42);
            return 27;
        })),
        27,
    );

    assert.deepEqual(
        ffi.call_with_42(wrap_cb_for_ffi((n) => {
            assert.deepEqual(n, 42);
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

    const start = performance.now();
    const ffi_long_running = ffi.long_running();
    const end = performance.now();
    const duration = end - start;
    assert(duration < 50.0, "Not more than 50 ms to perform the call");
    assert.deepEqual(
        await Promise.race(
            [
                ffi_long_running.then(() => "long_running"),
                new Promise(resolve => {
                    setTimeout(resolve, 10, "short_running");
                }),
            ]
        ),
        // FIXME: currently, support for `async_worker` under Wasm waits for
        // full completion of the function's body, so that here the "long
        // running" function has already been resolved.
        is_web ? "long_running" : "short_running",
    );
    assert.deepEqual(await ffi_long_running, 42);

	// Test conversion of bytes
	const site_id =  Uint8Array.from([0x00, 0x00,0x00, 0x00, 0x00, 0x00, 0x00, 0x05]);
    assert.equal(
		ffi.boxCStringIntoString(ffi.site_id(site_id)), // convert char_p::Box into JsString
		'[00, 00, 00, 00, 00, 00, 00, 05]'
    );

    // Test detached callback
    if (! is_web) {
        ffi.spinlock_aquire();
        let ret; ffi.call_detached(wrap_cb_for_ffi(() => {
            ffi.spinlock_aquire();
            ffi.spinlock_release();
            ret = 42;
        }));
        ffi.spinlock_release();
        while (typeof ret === 'undefined') {
            await new Promise((resolve, _) => {
                setTimeout(resolve, 0);
            });
        }
        assert.equal(ret, 42);
    }

    assert.deepEqual(await ffi.long_running_fut(Uint8Array.of(27)), 42 + 27);

    console.log('Js tests passed successfully ✅');
}

function wrap_cb_for_ffi(f) {
    return (send_ret, ...args) => {
        try {
            return send_ret(f(...args));
        } catch (e) {
            console.error(e);
        }
    };
}
