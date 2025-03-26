//! Representation

/// The possible bit-widths of an integral type.
///
/// You may use `as u8` to get a numeric value, but for the special
/// [`Self::PtrSized`] case, which has an unspecified integral discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub enum IntBitWidth {
    /// Whether the integer type is guaranteed to have the same bit-width
    /// as a pointer type.
    ///
    /// Note that for the sake of header simplicity and convenience, `safer-ffi`
    /// has been designed with an unsegmented architecture in mind, that is,
    /// one where indices and offsets use the same amount of bytes as pointers.
    ///
    /// That is, it assumes that `uintptr_t == uintoffset_t == size_t`.
    ///
    /// This variant has an unspecified `as u8` discriminant value.
    PtrSized,
    _8 = 8,
    _16 = 16,
    _32 = 32,
    _64 = 64,
    _128 = 128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub enum FloatBitWidth {
    _32 = 32,
    _64 = 64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Bool,
    Integer { signed: bool, bitwidth: IntBitWidth },
    Float { bitwidth: FloatBitWidth },
}
