local ffi = require("ffi")
local generated = require("generated")
local lib = ffi.load "libffi_tests"

function test_concat()
    local s1 = "Hello, "
    local s2 = "World!"
    local p = lib.concat(s1, s2);
    local res = ffi.string(p)
    lib.free_char_p(p);
    assert(res == s1 .. s2)
end

function test_max()
    local arr = { -27, -42, 9, -8 }
    local c_arr = ffi.new("int32_t[?]", #arr, arr)
    local c_slice = ffi.new("slice_ref_int32_t", c_arr, #arr)
    local p = lib.max(c_slice);
    assert(p ~= ffi.NULL)
    assert(p[0] == 9)
end

function test_max_empty()
    local arr = {}
    local c_arr = ffi.new("int32_t[?]", #arr, arr)
    local c_slice = ffi.new("slice_ref_int32_t", c_arr, #arr)
    local p = lib.max(c_slice);
    assert(p == ffi.NULL)
end

function test_foo()
    local foo = lib.new_foo()
    assert(lib.read_foo(foo) == 42)
    local called = false;
    lib.with_foo(function()
        assert(lib.read_foo(foo) == 42)
        called = true
    end)
    assert(called == true)
    lib.free_foo(foo)
    lib.free_foo(nil)
end

function test_constant()
    assert(lib.FOO == 42)
end

function test_currified_thing()
    assert(lib.returns_a_fn_ptr()(0x42) == 0x4200)
end

function test_enum_int_constant()
    assert(lib.TRIFORCE_DIN == 3)
    assert(lib.TRIFORCE_FARORE == 1)
    assert(lib.TRIFORCE_NARYU == 2)
end

function test_arrays_struct()
    local a = ffi.new("ArraysStruct_t")
    a.floats = ffi.new("float[3]", {7, 8, 9})
    a.sizes = ffi.new("size_t[5]", {5, 4, 3, 2, 1})
    a.dim_2 = ffi.new("uint8_t[2][1]", {{1}, {2}})
    a.dim_3 = ffi.new("uint8_t[3][2][1]", {{{1}, {1}}, {{2}, {2}}, {{3}, {3}}})

    assert(a.floats[0] == 7)
    assert(a.floats[1] == 8)
    assert(a.floats[2] == 9)

    assert(a.sizes[0] == 5)
    assert(a.sizes[1] == 4)
    assert(a.sizes[2] == 3)
    assert(a.sizes[3] == 2)
    assert(a.sizes[4] == 1)

    assert(a.dim_2[0][0] == 1)
    assert(a.dim_2[1][0] == 2)

    assert(a.dim_3[0][0][0] == 1)
    assert(a.dim_3[0][1][0] == 1)
    assert(a.dim_3[1][0][0] == 2)
    assert(a.dim_3[1][1][0] == 2)
    assert(a.dim_3[2][0][0] == 3)
    assert(a.dim_3[2][1][0] == 3)
end

function test_const_generics_struct()
    local a = ffi.new("SpecificConstGenericContainer_t")
    a.field1 = ffi.new("ConstGenericStruct_uint8_1_t", ffi.new("uint8_t[1]", {1}))
    a.field2 = ffi.new("ConstGenericStruct_uint8_2_t", ffi.new("uint8_t[2]", {1, 2}))
    a.field3 = ffi.new("ConstGenericStruct_uint16_3_t", ffi.new("uint16_t[3]", {1, 2, 3}))

    assert(a.field1.data[0] == 1)
    assert(a.field2.data[0] == 1)
    assert(a.field2.data[1] == 2)
    assert(a.field3.data[0] == 1)
    assert(a.field3.data[1] == 2)
    assert(a.field3.data[2] == 3)
end

function run_tests()
    local tests = {
        test_concat,
        test_max,
        test_max_empty,
        test_foo,
        test_constant,
        test_currified_thing,
        test_enum_int_constant,
        test_arrays_struct,
        test_const_generics_struct,
    }

    local passed = 0
    local failed = 0
    for _, test in ipairs(tests) do
        local status, err = pcall(test)
        if status then
            passed = passed + 1
        else
            failed = failed + 1
            print("Failed: " .. err)
        end
    end

    print(string.format("Passed: %d, Failed: %d", passed, failed))

    if failed > 0 then
        os.exit(1)
    end
end

run_tests()