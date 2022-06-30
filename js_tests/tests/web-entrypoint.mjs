import { run_tests } from './tests.mjs'
import load_ffi, * as ffi from '../pkg/js_tests.js';

async function run() {
    try {
        emit_console_logs_to_hmtl();

        await load_ffi();

        await run_tests({ ffi, assert, performance, is_web: true });
    } catch (err) {
        console.error(err);
    }
}

function emit_console_logs_to_hmtl() {
    const logger = document.getElementById("logger");
    console.old = console.log;
    console.log = function (...args) {
        var output = "";

        for (const arg of args) {
            output += "<span class=\"log-" + (typeof arg) + "\">";

            if (arg instanceof Error) {
                output += arg.toString();
            } else if (
                typeof arg === "object" &&
                typeof JSON === "object" &&
                typeof JSON.stringify === "function"
            ) {
                output += JSON.stringify(arg);
            } else {
                output += arg;
            }

            output += "</span>&nbsp;";
        }

        logger.innerHTML += output + "<br>";
        console.old.apply(undefined, args);
    };
    console.error = (...args) => {
        console.log("Error: '", ...args, "'");
    };
}

// Polyfill
function assert(condition, assertion) {
    if (!condition) {
        throw new Error(
            assertion
                ? `Assertion failed: '${assertion}'`
                : "Assertion failed"
        );
    }
}

function deepEqual(x, y) {
    if (x === y) {
        return true;
    } else if (
        (typeof x == "object" && x != null)
        &&
        (typeof y == "object" && y != null)
    )
    {
        if (Object.keys(x).length != Object.keys(y).length) {
            return false;
        }
        for (var prop in x) {
            if (!(
                y.hasOwnProperty(prop)
                &&
                deepEqual(x[prop], y[prop])
            ))
            {
                return false;
            }
        }
        return true;
    } else {
        return false;
    }
}

assert.equal = (lhs, rhs) => {
    assert(lhs === rhs, `${lhs} === ${rhs}`);
};
assert.deepEqual = (lhs, rhs) => {
    assert(deepEqual(lhs, rhs), `deepEqual(${lhs}, ${rhs})`);
};

run();
