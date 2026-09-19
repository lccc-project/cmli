use crate::{IdType, intern::Symbol};
use core::num::NonZero;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub enum OverflowKind {
    #[default]
    None,
    Unsigned,
    Signed,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct RelocSpan {
    /// The total number of bytes covered by the relocation, including bits that are referenced by the relocation, but not modified by it
    pub byte_width: u8,
    /// The first bit that will be modified by the relocation
    pub bit_offset: u8,
    /// The width of the modification, in bits
    pub bit_width: u8,
    /// The number of low order bits discarded from the address
    pub bit_shift: u8,
    /// The offset (from the address of the relocation) where the program counter value is calculated for a PC Relative relocation
    pub pcrel_offset: i8,
    /// Determines how to report overflow errors (if any).
    pub overflow_kind: OverflowKind,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl RelocSpan {
    pub const fn new() -> Self {
        Self {
            byte_width: 0,
            bit_offset: 0,
            bit_width: 0,
            pcrel_offset: 0,
            bit_shift: 0,
            overflow_kind: OverflowKind::None,
            __non_exhaustive: (),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, IdType)]
pub struct RelocationType(NonZero<u64>, u64);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RelocationKind {
    /// A relocation that performs no operation
    #[default]
    Null,
    /// A relocation against the absolute address
    Absolute(RelocSpan),
    Pcrel(RelocSpan),
    GotPcrel(RelocSpan),
    GotAbs(RelocSpan),
    Plt(RelocSpan),
    PltAbs(RelocSpan),
    GotDisp(RelocSpan),
    Tpoff(RelocSpan),
    GottpOff(RelocSpan),
    TlsGd(RelocSpan),
    TlsLd(RelocSpan),
    ImageRelative(RelocSpan),

    Relax(RelocationType),
    Other(RelocationType),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct RelocValue {
    pub sym: Option<Symbol>,
    pub addend: i64,
    pub kind: RelocationKind,
}
