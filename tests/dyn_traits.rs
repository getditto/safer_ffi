use ::safer_ffi::prelude::*;

#[derive_ReprC]
trait FfiFnMut {
    fn call (&mut self)
    ;
}

impl<T : FnMut()> FfiFnMut for T {
    fn call (&mut self)
    {
        self()
    }
}

#[ffi_export]
fn call (mut f: VirtualPtr<dyn '_ + FfiFnMut>)
{
    f.call();
    f.call();
}

#[derive_ReprC]
trait WithCallback {
    fn with(&self, scope: VirtualPtr<dyn '_ + FfiFnMut>)
    ;
}
