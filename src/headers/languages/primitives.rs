//! Representation

/// The possible bit-widths of an integral type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IntBitWidth {
    /// Whether the integer type is guaranteed to have the same bit-width
    /// as a pointer type.
    ///
    /// Note that for the sake of header simplicity and convenience, `safer-ffi`
    /// has been designed with an unsegmented architecture in mind, that is,
    /// one where indices and offsets use the same amount of bytes as pointers.
    ///
    /// That is, it assumes that `uintptr_t == uintoffset_t == size_t`.
    PointerSized,
    /// `c_int`.
    CInt,
    /// This is the only variant with a specified `as u8` discriminant value.
    Fixed(FixedIntBitWidth),
}

/// Fixed/platform-agnostic integral bit-width.
///
/// You may use `as u8` to get a numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub enum FixedIntBitWidth {
    _8 = 8,
    _16 = 16,
    _32 = 32,
    _64 = 64,
    _128 = 128,
}

impl FixedIntBitWidth {
    pub fn from_raw(num_bits: u8) -> Option<Self> {
        Some(match num_bits {
            | 8 => Self::_8,
            | 16 => Self::_16,
            | 32 => Self::_32,
            | 64 => Self::_64,
            | 128 => Self::_128,
            | _ => return None,
        })
    }
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
    CChar,
    Integer { signed: bool, bitwidth: IntBitWidth },
    Float { bitwidth: FloatBitWidth },
}
