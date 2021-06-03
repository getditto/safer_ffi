const ffi = require('.');
(async () => {
    await (await import('./main.mjs')).run_tests(ffi);
})()
