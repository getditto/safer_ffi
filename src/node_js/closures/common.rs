#![cfg_attr(rustfmt, rustfmt::skip)]

#[derive(Debug)]
pub
struct ArcClosureRawParts<CallFn> {
    pub data: *mut c_void,
    pub call: CallFn,
    pub release: unsafe extern "C" fn(_: *mut c_void),
    pub retain: unsafe extern "C" fn(_: *mut c_void),
}
