//! Dynamic provider API
//!
//!   - ("Mini" version: lifetime-infected `Any` case (_e.g._, to support ref-yielding) has been
//!     skipped.)
//!
//! # Rationale
//!
//! This is useful in a doubly-generic-but-separate scenario, wherein one party wants to _provide_
//! some kind of type, dynamically/not-known-ahead-of-time, to the other party, in case the latter
//! happens to be interested in it.
//!
//!   - That is, the double-party situation, with multiple implementors on each side, make it so it
//!     is not guaranteed for the exact two implementors to match as desired. Hence the
//!     `Option`-ness in the resulting APIs.
//!
//!   - The scenario is thus rather more akin to there being two complicit implementors, "hidden"
//!     amongst a bunch of neutral / unaware-of-the-other-side-specifics implementors.
//!
//!     They thus rely on an a pre-agreed-upon common "secret" —the _type_ of the value being
//!     provided/requested!—, for one party to be able to "smuggle" the value in a slot wherein the
//!     other party will know to look.
//!
//!       - given that the type itself is what acts as the id (and thus, "key") for the slot (to
//!         "open it"), it is *highly* advisable to involve newtypes with this pattern rather than
//!         bare, stdlib types.
//!
//!       - a future version of this pattern could involve an extra `key: &str` in all this, so as
//!         to require unicity of the `key, Type` pair (and maybe emit diagnostics when key match
//!         but not the types, for instance, or _vice-versa_).
//!
//! ## Rationale examples
//!
//! ### Fully dynamically-extensible state-passing in `serde`
//!
//! Typical example of such a doubly-generic-but-separate scenario would be a serialization
//! framework, such as `::serde`'s.
//!
//!   - on the one side, a variety of generic `impl {De,}Serialize` types;
//!   - on the other, a smaller but nonetheless plural amount of generic `impl {De,}Serializer`s.
//!
//! Now, imagine, for instance, there being a `{De,}Serializer` which can be flexible w.r.t. the
//! case involved, as in, it may sometimes be tweaked to *expect* `kebab-case`, and othertimes,
//! `snake_case`.
//!
//! Since it would be silly for the <code>impl {De,}Serialize</code> to be the one
//! picking the case involved in the {de,}serialization,
//!
//!   - <details><summary>right??</summary>
//!
//!     ![anakin-padme-meme](https://gist.github.com/user-attachments/assets/2aa7b14b-735c-4d06-a591-d040855043ee)
//!
//!     </details>
//!
//! it would be more typical for the `{De,}Serializer` to expose some API for the type in its
//! `impl {De,}Serialize` to know with which case convention it should provide its field names /
//! keys.
//!
//! And while _this_ very example seems so pervasive so as to hard-code it, we could, I guess,
//! envision future extensions to case conventions, only to be supported by specific pairs of
//! `{De,}Serializer`s and `impl {De,}Serialize` types.
//!
//!   - As an extremly contrived example, imagine there being a sponge-bob-json format, expecting a
//!     "sponge-bob-mocking case" convention.
//!
//!     Then, a sponge-bob-aware `impl {De,}Serialize` body could do:
//!
//!     ```rust
//!     # struct SerdeResult<S>(S);
//!     use ::core::marker::PhantomData;
//!     use ::safer_ffi::headers::provider::Provider;
//!
//!     struct KebabCase;
//!     struct SnakeCase;
//!     struct SpongeBobCase;
//!
//!     #[derive(Default)]
//!     enum CaseConvention {
//!         #[default]
//!         Kebab,
//!         Snake,
//!         SpongeBob,
//!     }
//!
//!     struct Thing;
//!
//!     trait Serializer {
//!         /// new kind of requirement, say.
//!         fn provider() -> impl Provider;
//!         // ...
//!     }
//!
//!     fn serialize<S: Serializer>(_: &Thing) -> SerdeResult<S> {
//!         let case_convention = || -> CaseConvention {
//!             let provider = &S::provider();
//!             if provider.request::<KebabCase>().is_some() {
//!                 return CaseConvention::Kebab;
//!             }
//!             if provider.request::<SnakeCase>().is_some() {
//!                 return CaseConvention::Snake;
//!             }
//!             if provider.request::<SpongeBobCase>().is_some() {
//!                 return CaseConvention::SpongeBob;
//!             }
//!             CaseConvention::default()
//!         }();
//!         // ...
//!         # unimplemented!("doc example")
//!     }
//!     ```
//!
//! ### Requesting extra "metadata" in `Error` types
//!
//! This is, in fact, what gave birth to this very design in Rust. An idea for the stdlib, that you
//! ought to be able to look up ("provider pattern error rust").
//!
//! # Example usage
//!
//! ```rust
//! use ::safer_ffi::headers::provider::Provider;
//! use ::safer_ffi::headers::provider::provide_with;
//!
//! trait PrettyPrint {
//!     fn pretty_print(
//!         &self,
//!         config: &impl Provider,
//!     ) -> String;
//! }
//!
//! /// By default, dynamic configs can be ignored (they represent optional extra metadata).
//! impl PrettyPrint for bool {
//!     fn pretty_print(
//!         &self,
//!         _: &impl Provider,
//!     ) -> String {
//!         self.to_string()
//!     }
//! }
//!
//! // But we could envision certain languages deciding to be aware of a certain specific knob:
//! #[derive(Default)]
//! enum Base {
//!     #[default]
//!     NinePlusOne,
//!     NinePlusSeven,
//! }
//!
//! impl PrettyPrint for u32 {
//!     fn pretty_print(
//!         &self,
//!         config: &impl Provider,
//!     ) -> String {
//!         match config.request::<Base>().unwrap_or_default() {
//!             | Base::NinePlusOne => format!("{self}"),
//!             | Base::NinePlusSeven => format!("{self:#x}"),
//!         }
//!     }
//! }
//!
//! // Notable `impl`ementor of `Provider`: `None`.
//! assert_eq!(true.pretty_print(&None), "true");
//! assert_eq!(42.pretty_print(&None), "42");
//! // Notable `impl`ementor of `Provider`: `impl Fn(&mut Request<'_>)`.
//! assert_eq!(
//!     42.pretty_print(&provide_with(|request| {
//!         request.give_if_requested::<Base>(|| Base::NinePlusSeven);
//!     })),
//!     "0x2a",
//! );
//! ```
//!
//! # Back to `safer-ffi`
//!
//! So, this pattern allows doubly-generic-but-protocol-separated parties to exchange some kind of
//! "metadata".
//!
//! It turns out, within `safer-ffi`, we do have this very pattern!
//!
//!   - <code>impl [CType]</code>s are our `impl Serialize`s;
//!       - or rather, the [`PhantomCType`]s, if we are to nitpick about `&self`.
//!   - And the [`HeaderLanguage`]s are our `Serializer`s.
//!
//! [CType]: `trait@crate::layout::CType`
//! [`PhantomCType`]: `super::languages::PhantomCType`
//! [`HeaderLanguage`]: `super::languages::HeaderLanguage`
//!
//! And what would be a typical example of _dynamic_ metadata to be exchanged between types and
//! languages?
//!
//! Well, I think that [C# marshalling annotations] are a perfect fit for it.
//!
//! [C# marshalling annotations]: https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.unmanagedtype
//!
//! Another use case is when a `HeaderLanguage` itself is being driven by a header-generation
//! call-site: we should now be able to envision the caller providing config knobs for at least
//! certain "header languages" to be able to pick up.

/// Convenience from-`impl Fn(…)` constructor of _ad-hoc_ <code>impl [Provider]</code>s.
#[allow(nonstandard_style)]
pub struct provide_with<F: Fn(&mut Request<'_>)>(pub F)
where
    Self: Provider;

impl<F: Fn(&mut Request<'_>)> Provider for provide_with<F> {
    #[inline]
    fn provide_to(
        &self,
        request: &mut Request<'_>,
    ) {
        self.0(request)
    }
}

/// Trivial `&`-transitivity of the trait, enabling, mainly for <code>dyn [Provider]</code>s, for
/// [`.request::<T>()`][`Provider::request()`] to be callable.
impl<P: ?Sized + Provider> Provider for &'_ P {
    #[inline]
    fn provide_to(
        &self,
        request: &mut Request<'_>,
    ) {
        P::provide_to(*self, request);
    }
}

/// `Provider` is implemented "for `None`" (by providing _nothing_, _i.e._, never calling any
/// [`.give_if_requested::<T>()`][`Request::give_if_requested()`]).
impl Provider for Option<::never_say_never::Never> {
    #[inline]
    fn provide_to(
        &self,
        _: &mut Request<'_>,
    ) {
    }
}

/// Rename used purely for documentation: we cannot do `Option<dyn Any>` (since `Option<>`
/// requires `Sized`), which is why we end up `Any`-erasing *everything*, including the
/// `Option` layer itself, but morally an `Option<dyn Any>` is basically what a [`Request`]
/// is about.
use ::core::any::Any as OptionAny;

// Morally, this type represents an `Option<dyn Any>`, initialized from some `None::<T>`,
// wherein the _requester_ having used this instance is looking to receive / be provided a value of
// type `T`.
/// Handle through which an implementor of [`Provider::provide_to()`] is expected to _provide_ /
/// give its value(s) of type `<T>`, through the
/// [`.give_if_requested::<T>()`][`Self::give_if_requested()`] method, to the
/// [`.request::<T>()`][`Provider::request()`]ers.
#[repr(transparent)]
pub struct Request<'lt>(
    /// We pre-reserve an invariant `'lt` param in this type should we end up "un-mini"-fying this
    /// module so as to support `'lt`-infected `T` types in the request, as in:
    /// `request::<&mut Vec<String>>()`.
    ///
    /// This ought to allow us to make that change in the future whilst remaining SemVer
    /// compatible.
    ::core::marker::PhantomData<fn(&()) -> &mut &'lt ()>,
    /// But, for now, settle for using `Any`, _i.e._, `'lt = 'static` in practice.
    dyn OptionAny,
);

impl Request<'_> {
    #[inline]
    fn wrap_mut<U: 'static>(requester_slot: &mut Option<U>) -> &mut Self {
        #[rustfmt::skip]
        return unsafe {
            // SAFETY: same layout of the pointee (thanks to `repr(transparent ≥ C)`),
            //         and usage of `as` casts makes this robust to whichever layout of the wide
            //         pointer is picked.
            //
            //         Finally, `Request` involves no extra validity nor safety invariants
            //         whatsoever.
            &mut *(
                ::core::ptr::addr_of_mut!(*requester_slot)
                  // : *mut Option<U>
                    as *mut dyn OptionAny
                    as *mut Self/*(dyn OptionAny)*/
            )
        };
    }

    /// Try to provide a value of type `T` to the [`.request()`]er.
    ///
    /// [`.request()`]: `Provider::request()`
    ///
    /// The closure is only invoked (and the value, provided) if and only if:
    ///
    ///  1. the [`.request()`] involved the same `T`.
    ///  1. a `T` value hasn't already been `put` in the [`Request`]
    ///
    ///     _i.e._, don't do:
    ///
    ///     ```rust
    ///     use ::safer_ffi::headers::provider::{Provider, Request};
    ///
    ///     struct Foo;
    ///     impl Provider for Foo {
    ///         fn provide_to(&self, request: &mut Request) {
    ///             // Provide `T = i32` once.
    ///             request.give_if_requested::<i32>(|| 42);
    ///             let mut called = false;
    ///             // Provide it a second time???
    ///             request.give_if_requested::<i32>(|| {
    ///                 called = true;
    ///                 27
    ///             });
    ///             // Should you ever do that, this second call will never have actually happened.
    ///             assert_eq!(called, false);
    ///         }
    ///     }
    ///
    ///     // request-site:
    ///     assert_eq!(Foo.request::</* T = */ i32>(), Some(42));
    ///     ```
    #[inline]
    pub fn give_if_requested<T: 'static + private::ObligatoryTurbofish<ItSelf = T>>(
        &mut self,
        f: impl FnOnce() -> T::ItSelf,
    ) -> &mut Self {
        // If the original `U` in `Option<U = impl 'static>` is `T`, then the request was
        // indeed interested in / requesting a value of type `T`.
        //
        // But if there is already one such value in the slot, we let whatever got there first
        // stay put.
        //
        //  whether downcast
        //   succeeded         whether the `Option<T>` itself is not yet filled with something.
        //     vvvv            vvvv
        if let Some(out @ &mut None) = self.1.downcast_mut::<Option<T>>() {
            //          ^^^^^^^^^^^
            //          same as doing `if out.is_none() {`, but with extra style points 😎.
            *out = Some(f());
        }
        self
    }
}

/// Assert `dyn`-compatibility.
impl dyn '_ + Provider {
    /// Convenience method for <code>dyn [Provider]</code>s:
    /// <code>(&self)[.request::\<T\>()][`Provider::request()`]</code>.
    pub fn dyn_request<T: 'static>(&self) -> Option<T> {
        (&self).request::<T>()
    }
}

/// Main `trait` for the [provider pattern][self], see the [module docs][self] for more info about
/// that.
///
///   - On the one side, callees / `impl`ementors are expected to provide (heh) an implementation of
///     the [`Self::provide_to()`] method.
///
///     This is achieved by calling
///     <code>request[.give_if_requested::\<T\>()]</code> with any number of choices of `<T>`.
///
///     [.give_if_requested::\<T\>()]: `Request::give_if_requested()`
///
///   - On the other side, call-sites are expected to using the convenience
///     [`.request::<T>()`][`Provider::request()`] method on <code>impl [Provider]</code> types.
///
/// # Examples
///
/// ## Basic
///
/// ```rust
/// use ::safer_ffi::headers::provider::Provider;
/// use ::safer_ffi::headers::provider::Request;
///
/// struct Foo;
///
/// impl Provider for Foo {
///     fn provide_to(
///         &self,
///         request: &mut Request<'_>,
///     ) {
///         request.give_if_requested::<i32>(|| 42);
///         request.give_if_requested::<bool>(|| true);
///         request.give_if_requested::<MyOwnSignal>(MyOwnSignal);
///     }
/// }
///
/// struct MyOwnSignal();
///
/// assert!(Foo.request::<i32>() == Some(42));
/// assert!(Foo.request::<bool>() == Some(true));
/// assert!(Foo.request::<MyOwnSignal>().is_some());
///
/// enum SomethingElse {}
/// assert!(Foo.request::<SomethingElse>().is_none());
/// ```
///
/// ## This trait is `dyn` compatible
///
/// ```rust
/// use ::safer_ffi::headers::provider::Provider;
/// use ::safer_ffi::headers::provider::provide_with;
///
/// fn demo(p: &dyn Provider) {
///     assert!(p.dyn_request::<i32>() == Some(42));
///     assert!(p.dyn_request::<u32>() == None);
/// }
///
/// demo(&provide_with(|request| {
///     request.give_if_requested::<i32>(|| 42);
/// }));
/// ```
#[::seal_the_deal::with_seals]
pub trait Provider {
    /// Method to be implemented by _the callee_ / the implementor / `Self`, by calling
    /// <code>request[.give_if_requested::\<T\>()]</code> with any number of choices of `<T>`.
    ///
    /// [.give_if_requested::\<T\>()]: `Request::give_if_requested()`
    fn provide_to(
        &self,
        request: &mut Request<'_>,
    );

    /// Convenience method for _callers_ dealing with some <code>impl [Provider]</code> type,
    /// for them to be able to _request_ / query / get the value of type `<T>` that this impl may
    /// have [`.give[n]_if_requested::<T>()`][`Request::give_if_requested()`].
    #[sealed]
    #[inline]
    fn request<T: 'static>(&self) -> Option<T>
    where
        Self: Sized,
    {
        let mut requester_slot = None::<T> {};
        self.provide_to(Request::wrap_mut(&mut requester_slot));
        requester_slot
    }
}

mod private {
    pub trait ObligatoryTurbofish {
        type ItSelf;
    }

    impl<T> ObligatoryTurbofish for T {
        type ItSelf = T;
    }
}
