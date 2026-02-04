use crate::{
    mach::{MachineMode, Register, RegisterSpec},
    traits::{AsId, AsRawId, IdType, Name},
};

use crate::instr::RegisterKind;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId, Name)]
pub enum X86Mode {
    Real,
    Protected16,
    Protected,
    Long,
}

impl X86Mode {
    pub fn largest_gpr(&self) -> u32 {
        match self {
            X86Mode::Real => 16,
            X86Mode::Protected16 => 16,
            X86Mode::Protected => 32,
            X86Mode::Long => 64,
        }
    }
}

impl AsId<MachineMode> for X86Mode {}

macro_rules! def_helper_arms {
    ($var:ident, $($capture:literal)* => $prefix:ident _) => {
        match $var {
            $($capture => ::core::concat!(::core::stringify!($prefix), ::core::stringify!($capture)),)*
            _ => unreachable!()
        }
    };
    ($var:ident, $($capture:literal)* => $prefix:ident _ $suffix:ident) => {
        match $var {
            $($capture => ::core::concat!(::core::stringify!($prefix), ::core::stringify!($capture), ::core::stringify!($suffix)),)*
            _ => unreachable!()
        }
    };
}

macro_rules! def_from_name_arms {
    ($var:ident, $begin:literal..$end:literal @ $($capture:literal)* => $class:ident ( $prefix:ident _)) => {
        match $var {
            $(::core::concat!(::core::stringify!($prefix), ::core::stringify!($capture)) if $begin<= $capture && $capture < $end=> return Self::$class($capture),)*
            _ => {}
        }
    };
    ($var:ident, $begin:literal..$end:literal @ $($capture:literal)* => $class:ident ( $prefix:ident _ $suffix:ident)) => {
        match $var {
            $(::core::concat!(::core::stringify!($prefix), ::core::stringify!($capture), ::core::stringify!($suffix)) if $begin<= $capture && $capture < $end=> return Self::$class($capture),)*
            _ => {}
        }
    };
}

macro_rules! define_x86_registers {
    {
        $vis:vis enum $name:ident {
            $($class:ident $(#[norex] $(@ $_norex_tt:tt)?)? [ $($names:ident),* $(, #$prefix:ident _ $($suffix:ident)? $begin:literal..$end:literal)?] $(($size:literal))? $(overlaps [$($overlap_var:ident),*])? = $kind:expr),* $(,)?
        }
    } => {
        #[derive(Copy, Clone, Hash, PartialEq, Eq, AsRawId)]
        #[repr(u8)]
        #[non_exhaustive]
        $vis enum $name {
            $($class (u8)),*
        }


        impl $name {

            #[doc(hidden)]
            pub const fn from_name_impl(name: &str) -> Self {
                match name {
                    $(
                        $(::core::stringify!($names) => Self::$class(${index()}),)*
                    )*

                    __x => {
                        $(
                            $(
                                def_from_name_arms!(__x, $begin..$end @ 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 => $class($prefix _ $($suffix)?));
                            )?
                        )*

                        panic!("Unknown Register")
                    }
                }
            }

            #[inline]
            pub fn from_name(name: &str) -> Self {
                Self::from_name_impl(name)
            }
        }


        impl Name for $name {
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$class(n) => {
                        match n $(& 0x7 $(# $_norex_tt)?)? {
                            $(${index()} => ::core::stringify!($names),)*
                            $($begin..$end => {
                                def_helper_arms!(n, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 => $prefix _ $($suffix)?)
                            })?
                            _ => "**Unknown Register**"
                        }
                    }),*
                }
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl $name {
            fn kind(&self) -> RegisterKind {
                match self {
                    $(Self::$class(_) => $kind),*
                }
            }

            fn size(&self, m: X86Mode) -> u32 {
                match self {
                    $(Self::$class(_) => ($($size,)? m.largest_gpr() / 8,).0),*
                }
            }

            fn overlaps_impl(&self, other: &X86Register) -> bool {
                match self {
                    $(Self::$class(__n) => {
                        match other {
                            $($(Self::$overlap_var(__n2) => __n == __n2,)*)?
                            _ => false
                        }
                    }),*
                }
            }

            pub const fn regno(&self) -> u8 {
                match self {
                    $(Self::$class(__n) => *__n $(& 7 $(@ $_norex_tt)?)?),*
                }
            }
        }
    }
}

define_x86_registers! {
    pub enum X86Register {
        Byte[al, cl, dl, bl, ah, ch, dh, bh] (1) = RegisterKind::GeneralPurpose,
        ByteRex[al, cl, dl, bl, spl, bpl, sil, dil, #r _ b 8..32] (1) overlaps [Word, Double, Quad] = RegisterKind::GeneralPurpose,
        Word[ax, cx, dx, bx, sp, bp, si, di, #r _ w 8..32] (2) overlaps [ByteRex, Double, Quad] = RegisterKind::GeneralPurpose,
        Double[eax, ecx, edx, ebx, esp, ebp, esi, edi, #r _ d 8..32] (4) overlaps [ByteRex, Word, Quad] = RegisterKind::GeneralPurpose,
        Quad [rax, rcx, rdx, rbx, rsp, rbp, rsi, rdi, #r _ 8..32] (8) overlaps [ByteRex, Word, Double] = RegisterKind::GeneralPurpose,
        Segment #[norex] [es, cs, ss, ds, fs, gs] (2) = RegisterKind::AddressSegment,
        St [, #st _ 0..8] (10) = RegisterKind::ScalarFp,
        Mmx #[norex] [, #mm _ 0..8] (8) = RegisterKind::VectorInt,
        Xmm [, #xmm _ 0..32] (16) overlaps [Ymm, Zmm] = RegisterKind::VectorAny,
        Ymm [, #ymm _ 0..32] (32) overlaps [Xmm, Zmm] = RegisterKind::VectorAny,
        Zmm [, #zmm _ 0..32] (64) overlaps [Xmm, Ymm] = RegisterKind::VectorAny,
        Tmm [, #tmm _ 0..32] (1024) = RegisterKind::VectorAny,
        Kreg [, #k _ 0..32] (8) = RegisterKind::VectorBit,
        Control [, #cr _ 0..32] = RegisterKind::System,
        Debug [, #dr _ 0..32] = RegisterKind::System,
        ExtControl [, #xcr _ 0..32] = RegisterKind::System,
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GprSize {
    Byte,
    Word,
    Double,
    Quad,
}

impl GprSize {
    pub const fn size(self) -> u32 {
        match self {
            Self::Byte => 1,
            Self::Word => 2,
            Self::Double => 4,
            Self::Quad => 8,
        }
    }

    pub const fn bits(self) -> u32 {
        match self {
            Self::Byte => 8,
            Self::Word => 16,
            Self::Double => 32,
            Self::Quad => 64,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XmmSize {
    Xmm,
    Ymm,
    Zmm,
}

impl X86Register {
    pub const fn promote_gpr(&self, new_size: GprSize) -> X86Register {
        match (self, new_size) {
            (Self::Byte(n), GprSize::Byte) => Self::Byte(*n),
            (Self::Byte(n), GprSize::Word) if *n < 4 => Self::Word(*n),
            (Self::Byte(n), GprSize::Double) if *n < 4 => Self::Double(*n),
            (Self::Byte(n), GprSize::Quad) if *n < 4 => Self::Quad(*n),
            (Self::ByteRex(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n), GprSize::Byte) => {
                if *n < 4 {
                    Self::Byte(*n)
                } else {
                    Self::ByteRex(*n)
                }
            }
            (Self::ByteRex(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n), GprSize::Word) => {
                Self::Word(*n)
            }
            (
                Self::ByteRex(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n),
                GprSize::Double,
            ) => Self::Double(*n),
            (Self::ByteRex(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n), GprSize::Quad) => {
                Self::Quad(*n)
            }
            _ => panic!("Not a GPR"),
        }
    }

    pub const fn gpr_size(&self) -> Option<GprSize> {
        match self {
            Self::Byte(_) | Self::ByteRex(_) => Some(GprSize::Byte),
            Self::Word(_) => Some(GprSize::Word),
            Self::Double(_) => Some(GprSize::Double),
            Self::Quad(_) => Some(GprSize::Quad),
            _ => None,
        }
    }

    pub const fn promote_xmm(&self, new_size: XmmSize) -> X86Register {
        match (self, new_size) {
            (Self::Xmm(n) | Self::Ymm(n) | Self::Zmm(n), XmmSize::Xmm) => Self::Xmm(*n),
            (Self::Xmm(n) | Self::Ymm(n) | Self::Zmm(n), XmmSize::Ymm) => Self::Ymm(*n),
            (Self::Xmm(n) | Self::Ymm(n) | Self::Zmm(n), XmmSize::Zmm) => Self::Zmm(*n),
            _ => panic!("Not a Vector Register"),
        }
    }

    pub const fn xmm_size(&self) -> Option<XmmSize> {
        match self {
            Self::Xmm(_) => Some(XmmSize::Xmm),
            Self::Ymm(_) => Some(XmmSize::Ymm),
            Self::Zmm(_) => Some(XmmSize::Zmm),
            _ => None,
        }
    }

    pub const fn valid_in_mode(&self, mode: X86Mode) -> bool {
        let is_64_bit = matches!(mode, X86Mode::Long);
        let allow_protection = !matches!(mode, X86Mode::Real); // No 

        if self.regno() > 8 && !is_64_bit {
            return false;
        }

        match self {
            X86Register::Ymm(_) | X86Register::Zmm(_) | X86Register::ExtControl(_)
                if !allow_protection =>
            {
                false
            }
            X86Register::Double(_) | X86Register::Debug(_) | X86Register::Control(_)
                if !allow_protection =>
            {
                false
            }
            X86Register::Quad(_) if !is_64_bit => false,
            _ => true,
        }
    }
}

impl AsId<Register> for X86Register {}

impl RegisterSpec for X86Register {
    type MachineMode = X86Mode;
    fn kind(&self) -> RegisterKind {
        self.kind()
    }

    fn size(&self, mode: X86Mode) -> u32 {
        self.size(mode)
    }

    fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Byte(n @ 0..4), Self::ByteRex(n2 @ 0..4))
            | (Self::ByteRex(n @ 0..4), Self::Byte(n2 @ 0..4)) => n == n2,
            (
                Self::Byte(n),
                Self::Word(n2 @ 0..4) | Self::Double(n2 @ 0..4) | Self::Quad(n2 @ 0..4),
            )
            | (
                Self::Word(n2 @ 0..4) | Self::Double(n2 @ 0..4) | Self::Quad(n2 @ 0..4),
                Self::Byte(n),
            ) => (*n & 3) == *n2,
            (Self::Mmx(m), Self::St(n)) | (Self::St(n), Self::Mmx(m)) => (*n & 7) == ((8 - *m) & 7),
            _ => self.overlaps_impl(other),
        }
    }
}

#[macro_export]
macro_rules! x86_registers {
    [$($reg:ident),* $(,)?] => {
        const { [$($crate::archs::x86::X86Register::from_name_impl(::core::stringify!($reg))),*]}
    }
}
