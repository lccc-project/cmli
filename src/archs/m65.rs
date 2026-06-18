use std::{hash::Hash, marker::{PhantomData, ConstParamTy}};

use crate::{instr::RegisterKind, mach::{Machine, MachineMode, MachineSpec, OneMachine, Opcode, Register, RegisterSpec, Regset, TargetFeatureSpec}, traits::{AsId, AsRawId, Name}};

#[cfg(feature = "xva")]
use crate::{compiler::CompilerSpec, xva::XvaCategory};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId)]
pub struct W65Mode(u32);

impl const AsId<MachineMode> for W65Mode {}

impl Name for W65Mode {
    fn name(&self) -> &'static str {
        "w65"
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, ConstParamTy)]
pub enum M65Kind {
    M6502,
    W65,
}

impl M65Kind {
    pub const fn gptr_size(self) -> u32 {
        match self {
            Self::M6502 => 8,
            Self::W65 => 16,
        }
    }

    pub const fn accum_size(self, mode: W65Mode) -> u32 {
        match self {
            Self::M6502 => 8,
            Self::W65 => 8 << ((mode.0 & 2) >> 1),
        }
    }

    pub const fn index_size(self, mode: W65Mode) -> u32 {
        match self {
            Self::M6502 => 8,
            Self::W65 => 8 << (mode.0 & 1),
        }
    }

    pub const fn has_w65(self) -> bool {
        matches!(self, Self::W65)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq,AsRawId)]
pub enum M65Register<const Kind: M65Kind> {
    A,
    X,
    Y,
    S,

    R(u8),
    Rw(u8),
    

    // w65 only registers
    B,
    D,
    K,

}

impl<const Kind: M65Kind> M65Register<Kind> {
    const ALL_REGISTERS: [Self; 31] = [Self::A, Self::X, Self::Y, Self::S, Self::B, Self::D, Self::K, 
        Self::R(0), Self::R(1), Self::R(2), Self::R(3), Self::R(4), Self::R(5), Self::R(6), Self::R(7),
        Self::Rw(0), Self::Rw(1), Self::Rw(2), Self::Rw(3), Self::Rw(4), Self::Rw(5), Self::Rw(6), Self::Rw(7),
        Self::Rw(8), Self::Rw(9), Self::Rw(10), Self::Rw(11), Self::Rw(12), Self::Rw(13), Self::Rw(14), Self::Rw(15),
    ];
}

impl<const Kind: M65Kind> M65Register<Kind> {
    pub const fn kind(&self) -> RegisterKind {
        match self {
            M65Register::X |
            M65Register::Y  |
            M65Register::R(_) |
            M65Register::Rw(_) |
            M65Register::A => RegisterKind::GeneralPurpose,
            M65Register::S |
            M65Register::D => RegisterKind::AddressOnly,
            M65Register::B |
            M65Register::K => RegisterKind::AddressSegment,
        }
    }
}

impl<const Kind: M65Kind> Name for M65Register<Kind> {
    fn name(&self) -> &'static str {
        match self {
            M65Register::A => "A",
            M65Register::X => "X",
            M65Register::Y => "Y",
            M65Register::S => "S",
            M65Register::R(r) => regno_to_static_name!(*r => "r"),
            M65Register::Rw(r) => regno_to_static_name!(*r => "rw"),
            M65Register::D => "D",
            M65Register::B => "B",
            M65Register::K => "K",
        }
    }
}

impl<const Kind: M65Kind> const AsId<Register> for M65Register<Kind> {}



impl<const Kind: M65Kind> RegisterSpec for M65Register<Kind> {
    type MachineMode = W65Mode;
    
    fn kind(&self) -> crate::instr::RegisterKind {
        self.kind()
    }
    
    fn size(&self, mode: Self::MachineMode) -> u32 {
        match self {
            M65Register::A => Kind.accum_size(mode),
            M65Register::X |
            M65Register::Y => Kind.index_size(mode),
            M65Register::S => Kind.gptr_size(),
            M65Register::R(_) => 4,
            M65Register::Rw(_) => 2,
            M65Register::B => 1,
            M65Register::D => 2,
            M65Register::K => 1,
        }
    }
    
    #[cfg(feature = "xva")]
    fn category(&self, mode: Self::MachineMode) -> crate::xva::XvaCategory {
        match self.kind() {
            RegisterKind::GeneralPurpose => XvaCategory::Int,
            kind => XvaCategory::Custom(kind)
        }
    }
    
    fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (a,b) if a==b => true,
            (M65Register::R(rn), M65Register::Rw(wn))|(M65Register::Rw(wn), M65Register::R(rn)) => *rn == (*wn >> 1),
            _ => false,
        }
    }
    
    fn from_bit(bit: u32, _: Self::MachineMode) -> Option<Self> {
        match bit {
            0 => Some(Self::A),
            1 => Some(Self::X),
            2 => Some(Self::Y),
            v @ (8..16) => Some(Self::R((v&7) as u8)),
            v @ 16..32 => Some(Self::Rw((v & 15) as u8)),
            _ => None
        }
    }
    
    fn regmap_bit(self) -> Option<u32> {
        match self {
            Self::A => Some(0),
            Self::X => Some(1),
            Self::Y => Some(2),
            Self::R(v) => Some(8 | v as u32),
            Self::Rw(v) => Some(16 | v as u32),
            _ => None
        }
    }
    
    fn supported_registers(_: &crate::mach::FeatureSet, _: Self::MachineMode) -> crate::mach::Regset {
        Regset::from_registers(core::iter::chain([Self::A, Self::X, Self::Y, Self::S], (0..8).map(Self::R))
            .chain((0..16).map(Self::Rw))
            .chain([Self::K, Self::D, Self::B].into_iter().filter(|_| const { Kind.has_w65() }))
        )
    }

    
}

macro_rules! m65_instructions {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident <const $kind:ident: $ty:ty> {
            $($(#[$instr_meta:meta])*  $instr_name:ident $([$global_mode:pat])? ($mnemonic:literal) {
                $([$($operand:pat),* $(,)?] $($mode:pat)? => $opcode:literal),+ $(,)?
            })*
        }
    } => {

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId)]
        $(#[$meta])*
        $vis enum $name <const $kind: $ty>{
            $(
                #[doc = ::core::concat!("The `", $mnemonic, "` instruction")]  
                $(#[$instr_meta])* 
                $instr_name
            ),*
        }

        impl <const $kind: $ty> $name <$kind> {
            const ALL_OPCODES: [Self; ${count($instr_name)}] = [$(Self::$instr_name),*];
        }

        impl <const $kind: $ty> const $crate::traits::AsId<$crate::mach::Opcode> for $name <$kind> {}

        impl<const $kind: $ty> $crate::traits::Name for $name <$kind> {
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$instr_name => $mnemonic),*
                }
            }
        }

    };
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ImmediateSize {
    Byte,
    Word,
    Acc,
    Idx,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum IndexReg {
    X,
    Y,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum M65Operand {
    Immediate(ImmediateSize),
    Abs(IndexReg),
    Abs24,

}

m65_instructions! {
    /// Instructions for M6502
    pub enum M65Opcode <const Kind: M65Kind> {
        Brk ("BRK") {
            [Immediate(Byte)] => 0x00
        }
    }
}

type W65Opcode = M65Opcode<{M65Kind::W65}>;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Name)]
pub enum M65TargetFeature {}

impl TargetFeatureSpec for M65TargetFeature {
    fn feature_to_bit(&self) -> u32 {
        match *self {}
    }

    fn from_name(name: &str) -> Option<Self> {
        None
    }

    fn feature_from_bit(bit: u32) -> Option<Self> {
        None
    }
}

pub struct M65Machine<const Kind: M65Kind>;

impl<const Kind: M65Kind> MachineSpec for M65Machine<Kind> {
    type Opcode = M65Opcode<Kind>;

    const OPCODES: &[Opcode] = as_id_array!(M65Opcode::<Kind>::ALL_OPCODES => Opcode);

    type Register = M65Register<Kind>;

    const REGISTERS: &[Register] = as_id_array!(M65Register::<Kind>::ALL_REGISTERS => Register);

    type MachineMode = W65Mode;

    const MACH_MODES: &[MachineMode] = as_id_array!([W65Mode(0o0), W65Mode(0o1), W65Mode(0o2), W65Mode(0o3), W65Mode(0o7)] => MachineMode);


    type TargetFeature = M65TargetFeature;


    fn name(&self) -> &'static str {
        match Kind {
            M65Kind::M6502 => "m6502",
            M65Kind::W65 => "w65",
        }
    }

    #[cfg(feature = "xva")]
    fn as_compiler(&self) -> Option<&dyn crate::compiler::CheckCompiler<Machine=Self>> {
        core::any::try_as_dyn(self)
    }
}

