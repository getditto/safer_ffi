#pragma warning disable IDE0044, IDE0049, IDE0055, IDE1006,
#pragma warning disable SA1004, SA1008, SA1023, SA1028,
#pragma warning disable SA1121, SA1134,
#pragma warning disable SA1201,
#pragma warning disable SA1300, SA1306, SA1307, SA1310, SA1313,
#pragma warning disable SA1500, SA1505, SA1507,
#pragma warning disable SA1600, SA1601, SA1604, SA1605, SA1611, SA1615, SA1649,

namespace {NameSpace} {{
using System;
using System.Runtime.InteropServices;

public unsafe partial class Ffi {{
#if IOS
    private const string RustLib = "{RustLib}.framework/{RustLib}";
#else 
    private const string RustLib = "{RustLib}";
#endif
}}
