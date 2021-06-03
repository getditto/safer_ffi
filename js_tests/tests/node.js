const ffi = require('.');
const { performance, } = require('perf_hooks');
(async () => {
    await (await import('./main.mjs')).run_tests(ffi, performance);
})()
