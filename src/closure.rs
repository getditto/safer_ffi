use ::std::{
    hint,
    ops::Not,
    os::raw::c_void,
};

/// Box<dyn 'static + Sync + Send + Fn()>
#[repr(C)]
pub
struct CBoxFn {
    /// Box<Erased>
    ptr: *mut c_void,

    call: Option<unsafe extern "C" fn (*const c_void)>,

    free: Option<unsafe extern "C" fn (*mut c_void)>,
}

/// Safety: This is Box<F>, and the constructor requires a `F : Send` bound
unsafe impl Send for CBoxFn {}
/// Safety: This is Box<F>, and the constructor requires a `F : Sync` bound
unsafe impl Sync for CBoxFn {}

impl<F> From<Box<F>> for CBoxFn
where
    F : Fn(),
    F : Send + Sync + 'static,
{
    #[inline]
    fn from (f: Box<F>) -> Self
    {
        unsafe extern "C" fn free<F> (ptr: *mut c_void)
        where
            F : Fn(),
            F : Send + Sync + 'static,
        {
            drop::<Box<F>>(Box::from_raw(ptr.cast()));
        }

        unsafe extern "C" fn call<F> (ptr: *const c_void)
        where
            F : Fn(),
            F : Send + Sync + 'static,
        {
            let f: &F = &*ptr.cast();
            f()
        }
        Self {
            ptr: Box::into_raw(f).cast(),
            free: Some(free::<F>),
            call: Some(call::<F>),
        }
    }
}

impl Drop for CBoxFn {
    fn drop (self: &'_ mut Self)
    {
        let &mut Self { ptr, free, .. } = self;
        debug_assert!(ptr.is_null().not());
        let free = if let Some(it) = free { it } else {
            if cfg!(debug_assertions) {
                unreachable!("`free == NULL`");
            } else {
                unsafe { hint::unreachable_unchecked() }
            }
        };
        unsafe {
            free(ptr);
        }
    }
}

impl CBoxFn {
    #[inline]
    pub fn call (self: &'_ Self)
    {
        let &Self { ptr, call, .. } = self;
        debug_assert!(ptr.is_null().not());
        let call = if let Some(it) = call { it } else {
            if cfg!(debug_assertions) {
                unreachable!("`call == NULL`");
            } else {
                unsafe { hint::unreachable_unchecked() }
            }
        };
        unsafe {
            call(ptr);
        }
    }
}
