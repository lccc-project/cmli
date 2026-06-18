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
    pub byte_width: u8,
    pub bit_offset: u8,
    pub bit_width: u8,
    pub bit_shift: u8,
    pub pcrel_offset: u8,
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
            __non_exhaustive: ()
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, IdType)]
pub struct RelocationType(NonZero<u64>, u64);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RelocationKind {
    #[default]
    Null,
    Absolute(RelocSpan),
    Pcrel(RelocSpan),
    GotPcrel(RelocSpan),
    Plt(RelocSpan),
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