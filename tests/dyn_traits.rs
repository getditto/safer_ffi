use ::safer_ffi::prelude::*;

#[derive_ReprC(dyn)]
trait FfiFnMut {
    fn call(&mut self);
}

impl<T: FnMut()> FfiFnMut for T {
    fn call(&mut self) {
        self()
    }
}

#[ffi_export]
fn call(mut f: VirtualPtr<dyn '_ + FfiFnMut>) {
    f.call();
    f.call();
}

#[derive_ReprC(dyn)]
trait WithCallback {
    fn with(
        &self,
        scope: VirtualPtr<dyn '_ + FfiFnMut>,
    );
}

#[derive_ReprC(dyn)]
trait Example {
    fn method(&self);
}

fn _example<'r, T: 'r + Example>(
    owned: T,
    r: &'r T,
    m: &'r mut T,
) {
    let owned: VirtualPtr<dyn 'r + Example> = Box::new(owned).into();
    owned.method();
    let r: VirtualPtr<dyn 'r + Example> = r.into();
    r.method();
    let m: VirtualPtr<dyn 'r + Example> = m.into();
    m.method();
}

#[derive_ReprC(dyn, Clone)]
trait Cloneable {
    fn method(&self);
}

fn _cloneable<'r, T: 'r + Cloneable + Clone>(
    owned: T,
    r: &'r T,
) {
    let owned: VirtualPtr<dyn 'r + Cloneable> = Box::new(owned).into();
    let owned2: VirtualPtr<dyn 'r + Cloneable> = owned.clone();
    owned.method();
    owned2.method();
    let r: VirtualPtr<dyn 'r + Cloneable> = r.into();
    let r2: VirtualPtr<dyn 'r + Cloneable> = r.clone();
    r.method();
    r2.method();
}
