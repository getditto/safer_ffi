const ffi = require('./rust.node');
const { performance } = require('perf_hooks');
const assert = require('assert').strict;

(async () => {
    const { run_tests } = await import('./tests.mjs');
    await run_tests({ ffi, performance, assert, is_web: false });
})()
