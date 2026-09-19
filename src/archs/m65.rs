use std::{
    hash::Hash, marker::{ConstParamTy, PhantomData}, num::NonZero,
};

use sym_gen::Symbol;

use crate::{
    instr::{Address, AddressKind, Instruction, MemoryOperand, Operand, RegisterKind, RelocSym}, mach::{
        Machine, MachineMode, MachineSpec, OneMachine, Opcode, Register, RegisterSpec, Regset,
        TargetFeatureSpec,
    }, traits::{AsId, AsRawId, IdType, Name}, xva::{self, XvaConst},
};

#[cfg(feature = "xva")]
use crate::{compiler::CompilerSpec, mach::CompilerWrapper, xva::XvaCategory};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId)]
pub struct W65Mode(pub u64);

const impl AsId<MachineMode> for W65Mode {}

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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId)]
pub enum M65Register<const Kind: M65Kind> {
    Ab,
    Xb,
    Yb,
    Aw,
    Xw,
    Yw,
    S,

    R(u8),
    Rw(u8),

    // w65 only registers
    B,
    D,
    K,
}

impl<const Kind: M65Kind> M65Register<Kind> {
    const ALL_REGISTERS: [Self; 34] = [
        Self::Ab,
        Self::Xb,
        Self::Yb,
        Self::Aw,
        Self::Xw,
        Self::Yw,
        Self::S,
        Self::B,
        Self::D,
        Self::K,
        Self::R(0),
        Self::R(1),
        Self::R(2),
        Self::R(3),
        Self::R(4),
        Self::R(5),
        Self::R(6),
        Self::R(7),
        Self::Rw(0),
        Self::Rw(1),
        Self::Rw(2),
        Self::Rw(3),
        Self::Rw(4),
        Self::Rw(5),
        Self::Rw(6),
        Self::Rw(7),
        Self::Rw(8),
        Self::Rw(9),
        Self::Rw(10),
        Self::Rw(11),
        Self::Rw(12),
        Self::Rw(13),
        Self::Rw(14),
        Self::Rw(15),
    ];
}

pub type W65Register = M65Register::<{M65Kind::W65}>;

impl<const Kind: M65Kind> M65Register<Kind> {
    pub const fn kind(&self) -> RegisterKind {
        match self {
            M65Register::Xb
            | M65Register::Yb
            | M65Register::R(_)
            | M65Register::Rw(_)
            | M65Register::Ab
            | M65Register::Xw
            | M65Register::Yw
            | M65Register::Aw => RegisterKind::GeneralPurpose,
            M65Register::S | M65Register::D => RegisterKind::AddressOnly,
            M65Register::B | M65Register::K => RegisterKind::AddressSegment,
        }
    }

    pub fn to_addr(&self) -> Option<Address> {
        match self {
            Self::R(v) => {
                Some(Address { segment: None, 
                    base: Some(Register::new(Self::D)), 
                    index: None, 
                    scale: nzlit!(1), 
                    sym: Some(RelocSym { 
                        sym: Symbol::intern(format!("__r{v}")), 
                        kind: AddressKind::Default 
                    }), 
                    disp: None, 
                    rel: false 
                })
            }
            Self::Rw(v) => {
                Some(Address { segment: None, 
                    base: Some(Register::new(Self::D)), 
                    index: None, 
                    scale: nzlit!(1), 
                    sym: Some(RelocSym { 
                        sym: Symbol::intern(format!("__r{}", (*v) >> 1)), 
                        kind: AddressKind::Default 
                    }), 
                    disp: NonZero::new(((*v as i64) & 1) << 1), 
                    rel: false 
                })
            }
            _ => None
        }
    }
}

impl<const Kind: M65Kind> Name for M65Register<Kind> {
    fn name(&self) -> &'static str {
        match self {
            M65Register::Ab => "A",
            M65Register::Xb => "X",
            M65Register::Yb => "Y",
            M65Register::Aw => "C",
            M65Register::Xw => "Xw",
            M65Register::Yw => "Yw",
            M65Register::S => "S",
            M65Register::R(r) => regno_to_static_name!(*r => "r"),
            M65Register::Rw(r) => regno_to_static_name!(*r => "rw"),
            M65Register::D => "D",
            M65Register::B => "B",
            M65Register::K => "K",
        }
    }
}

const impl<const Kind: M65Kind> AsId<Register> for M65Register<Kind> {}

impl<const Kind: M65Kind> RegisterSpec for M65Register<Kind> {
    type MachineMode = W65Mode;

    fn kind(&self) -> crate::instr::RegisterKind {
        self.kind()
    }

    fn size(&self, mode: Self::MachineMode) -> u32 {
        match self {
            M65Register::Ab |
            M65Register::Xb | M65Register::Yb => 1,
            M65Register::Aw | M65Register::Xw | M65Register::Yw => 2,
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
            kind => XvaCategory::Custom(kind),
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (a, b) if a == b => true,
            (M65Register::Ab, M65Register::Aw) | (M65Register::Aw, M65Register::Ab) => true,
            (M65Register::Xb, M65Register::Xw) | (M65Register::Xw, M65Register::Xb) => true,
            (M65Register::Yb, M65Register::Yw) | (M65Register::Yw, M65Register::Yb) => true,
            (M65Register::R(rn), M65Register::Rw(wn))
            | (M65Register::Rw(wn), M65Register::R(rn)) => *rn == (*wn >> 1),
            _ => false,
        }
    }

    fn from_bit(bit: u32, _: Self::MachineMode) -> Option<Self> {
        match bit {
            0 => Some(Self::Ab),
            1 => Some(Self::Xb),
            2 => Some(Self::Yb),
            4 => Some(Self::Aw),
            5 => Some(Self::Xw),
            6 => Some(Self::Yw),
            7 => Some(Self::B),
            v @ (8..16) => Some(Self::R((v & 7) as u8)),
            v @ 16..32 => Some(Self::Rw((v & 15) as u8)),
            _ => None,
        }
    }

    fn regmap_bit(self) -> Option<u32> {
        match self {
            Self::Ab => Some(0),
            Self::Xb => Some(1),
            Self::Yb => Some(2),
            Self::Aw => Some(4),
            Self::Xw => Some(5),
            Self::Yw => Some(6),
            Self::B => Some(7),
            Self::R(v) => Some(8 | v as u32),
            Self::Rw(v) => Some(16 | v as u32),
            _ => unreachable!("Illegal Register: {self:?}"),
        }
    }

    fn supported_registers(
        _: &crate::mach::FeatureSet,
        _: Self::MachineMode,
    ) -> crate::mach::Regset {
        Regset::from_registers(
            core::iter::chain([Self::Ab, Self::Xb, Self::Yb], (0..8).map(Self::R))
                .chain((0..16).map(Self::Rw))
                .chain(
                    [Self::Aw, Self::Xw, Self::Yw, Self::B]
                        .into_iter()
                        .filter(|_| const { Kind.has_w65() }),
                ),
        )
    }
}

macro_rules! m65_instructions {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident <const $kind:ident: $ty:ty> {
            $($(#[$instr_meta:meta])*  $instr_name:ident $([$global_mode:pat])? ($mnemonic:literal) $(= $base_opc:literal)? {
                $([$($operand:pat),* $(,)?] $($mode:pat)? => $opcode:literal),* $(,)?
                $(![$($poperand:pat),* $(,)?] $($pmode:pat)? => $rinstr_name:ident [$($ropr:expr),* $(,)?],)*
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

        const impl <const $kind: $ty> $crate::traits::AsId<$crate::mach::Opcode> for $name <$kind> {}

        impl<const $kind: $ty> $crate::traits::Name for $name <$kind> {
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$instr_name => $mnemonic),*
                }
            }
        }

        impl <const $kind: $ty> $name <$kind> {
            pub fn valid(&self) -> bool {
                use M65Kind::*;
                match self {
                    $(Self:: $instr_name => {
                        $(matches!($kind, $global_mode) &&)? true
                    })*
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

impl IndexReg {
    pub const fn into_byte_reg<const Kind: M65Kind>(self) -> M65Register<Kind> {
        match self {
            IndexReg::X => M65Register::Xb,
            IndexReg::Y => M65Register::Yb,
        }
    }

    pub const fn into_word_reg(self) -> W65Register {
        match self {
            IndexReg::X => M65Register::Xw,
            IndexReg::Y => M65Register::Yw,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum M65Operand<const Kind: M65Kind> {
    Immediate(ImmediateSize),
    Abs(Option<IndexReg>),
    Abs24(Option<IndexReg>),
    Rel8,
    Rel16,
    Direct(Option<IndexReg>),
    Ind(Option<IndexReg>),
    IndirectDp(Option<IndexReg>),
    IndirectLong(Option<IndexReg>),
    Indirect24,
    BlockOp,
    RegA,
    RegX,
    RegY,
    RegS,
    Isy,
    Stack,
}

m65_instructions! {
    /// Instructions for M6502
    pub enum M65Opcode <const Kind: M65Kind> {
        Brk ("BRK") {
            [Immediate(Byte)] => 0x00,
            ![] => Brk [Operand::Immediate(0)],
        }
        Cop ("COP") {
            [Immediate(Byte)] => 0x02
        }
        Ora ("ORA") = 0x00 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F,
        }
        Adc ("ADC") = 0x60{
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F
        }
        Sbc ("SBC") = 0xE0 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F
        }
        Cmp ("CMP") = 0xC0 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F
        }
        Cpx ("CPX") {
            [Immediate(ImmediateSize::Idx)] => 0xE0,
            [Direct(None)] => 0xE4,
            [Absolute(None)] => 0xEC
        }
        Cpy ("CPY") {
            [Immediate(ImmediateSize::Idx)] => 0xC0,
            [Direct(None)] => 0xC4,
            [Absolute(None)] => 0xCC
        }
        Inc ("INC") {
            [] => 0x1A,
            [Direct(None)] => 0xE6,
            [Absolute(None)] => 0xEE,
            [Direct(Some(X))] => 0xF6,
            [Absolute(Some(X))] => 0xFE,
            ![RegA] => Inc [],
            ![RegX] => Inx [],
            ![RegY] => Iny [],
        }
        Dec ("DEC") {
            [] => 0x3A,
            [Direct(None)] => 0xC6,
            [Absolute(None)] => 0xCE,
            [Direct(Some(X))] => 0xD6,
            [Absolute(Some(X))] => 0xDE,
            ![RegA] => Dec [],
            ![RegX] => Dex [],
            ![RegY] => Dey [],
        }

        Inx ("INX") {
            [] => 0xE8
        }
        Iny ("INY") {
            [] => 0xC8
        }
        Dex ("DEX") {
            [] => 0xCA
        }
        Dey ("DEY") {
            [] => 0x88
        }

        And ("AND") = 0x20 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F,
        }

        Eor ("eor") = 0x40 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F,
        }

        Bit ("bit") {
            [Direct(None)] => 0x24,
            [Absolute(None)] => 0x2C,
            [Direct(Some(X))] => 0x34,
            [Absolute(Some(X))] => 0x3C,
            [Immediate(ImmediateSize::Acc)] => 0x89,
        }

        Tsb ("tsb") {
            [Direct(None)] => 0x04,
            [Absolute(None)] => 0x0C,
        }

        Trb ("trb") {
            [Direct(None)] => 0x14,
            [Absolute(None)] => 0x1C,
        }

        Asl ("asl") = 0x00{
            [Direct(None)] => 0x06,
            [] => 0x0A,
            [Absolute(None)] => 0x0E,
            [Direct(Some(X))] => 0x16,
            [Absolute(Some(X))] => 0x1E,
        }

        Lsr ("lsr") = 0x40 {
            [Direct(None)] => 0x06,
            [] => 0x0A,
            [Absolute(None)] => 0x0E,
            [Direct(Some(X))] => 0x16,
            [Absolute(Some(X))] => 0x1E,
        }

        Rol ("rol") = 0x20 {
            [Direct(None)] => 0x06,
            [] => 0x0A,
            [Absolute(None)] => 0x0E,
            [Direct(Some(X))] => 0x16,
            [Absolute(Some(X))] => 0x1E,
        }

        Ror ("ror") = 0x60 {
            [Direct(None)] => 0x06,
            [] => 0x0A,
            [Absolute(None)] => 0x0E,
            [Direct(Some(X))] => 0x16,
            [Absolute(Some(X))] => 0x1E,
        }

        Bra ("bra") {
            [Rel8] => 0x80,
            [Rel16] W65 => 0x82,
        }

        Bcc ("bcc") {
            [Rel8] => 0x90
        }

        Bcs ("bcs") {
            [Rel8] => 0xB0
        }

        Beq ("beq") {
            [Rel8] => 0xF0
        }

        Bmi ("bmi") {
            [Rel8] => 0x30
        }

        Bne ("bne") {
            [Rel8] => 0xD0
        }

        Bpl ("bpl") {
            [Rel8] => 0x10
        }

        Bvc ("bvc") {
            [Rel8] => 0x80
        }

        Bvs ("bvs") {
            [Rel8] => 0x70
        }

        Jmp ("jmp") {
            [Abs(None)] => 0x4C,
            [Abs24(None)] W65 => 0x5C,
            [Indirect(None)] => 0x6C,
            [Indirect(Some(X))] => 0x7C,
            [IndirectLong] W65 => 0xDC,
        }
        Jsl [W65] ("jsl") {
            [Abs24(None)] => 0x22,
        }
        Jsr ("jsr") {
            [Abs(None)] => 0x20,
            [Indirect(Some(X))] => 0xFC,
        }

        Rtl [W65] ("rtl") {
            [] => 0x6B
        }

        Rts ("rts") {
            [] => 0x60
        }
        Rti ("rti") {
            [] => 0x40
        }

        Clc ("clc") {
            [] => 0x18
        }

        Cld ("cld") {
            [] => 0xD8
        }

        Cli ("cli") {
            [] => 0x58,
        }

        Clv ("clv") {
            [] => 0xB8
        }

        Sec ("sec") {
            [] => 0x38
        }

        Sed ("sed") {
            [] => 0xF8
        }

        Sei ("sei") {
            [] => 0x78
        }

        Rep ("rep") {
            [Immediate(Byte)] => 0xC2
        }

        Sep ("sep") {
            [Immediate(Byte)] => 0xE2
        }

        Lda ("lda") = 0xA0 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F,

            ![RegX] => Txa [],
            ![RegY] => Txy [],
        }

        Sta ("sta") = 0x80 {
            [Indirect(Some(X))] => 0x01,
            [Stack] => 0x03,
            [Direct(None)] => 0x05,
            [IndirectLong(None)] W65 => 0x07,
            [Immediate(ImmediateSize::Acc)] => 0x09,
            [Abs(None)] => 0x0D,
            [Abs24(None)] => 0x0F,
            [IndirectDp(Some(Y))] => 0x11,
            [IndirectDp(None)] => 0x12,
            [Isy] => 0x13,
            [Direct(Some(X))] => 0x15,
            [IndirectLong(Some(Y))] => 0x17,
            [Abs(Some(Y))] => 0x19,
            [Abs(Some(X))] => 0x1D,
            [Abs24(Some(X))] => 0x1F,

            ![RegX] => Tax [],
            ![RegY] => Tay [],
        }

        Ldx ("ldx") = 0xA2 {
            [Immediate(ImmediateSize::Idx)] => 0x00,
            [Direct(None)] => 0x04,
            [Abs(None)] => 0x0C,
            [Direct(Some(Y))] => 0x14,
            [Abs(Some(Y))] => 0x1C,
        }

        Ldy ("ldy") = 0xA0 {
            [Immediate(ImmediateSize::Idx)] => 0x00,
            [Direct(None)] => 0x04,
            [Abs(None)] => 0x0C,
            [Direct(Some(X))] => 0x14,
            [Abs(Some(X))] => 0x1C,
        }

        Stx ("stx") = 0x82 {
            [Direct(None)] => 0x04,
            [Abs(None)] => 0x0C,
            [Direct(Some(Y))] => 0x14,
        }

        Sty ("sty") = 0x80 {
            [Direct(None)] => 0x04,
            [Abs(None)] => 0x0C,
            [Direct(Some(X))] => 0x14,
        }

        Stz ("stz") {
            [Direct(None)] => 0x64,
            [Abs(None)] => 0x9C,
            [Direct(Some(X))] => 0x74,
            [Abs(Some(X))] => 0x9E,
        }

        Mvn [W65] ("mvn") {
            [BlockOp, BlockOp] => 0x54,
        }

        Mvp [W65] ("mvp") {
            [BlockOp, BlockOp] => 0x44,
        }

        Nop ("nop") {
            [] => 0xEA
        }
        Wdm ("wdm") {
            [Immediate(ImmediateSize::Byte)] => 0x42,
        }
        
        Pea ("pea") {
            [Immediate(ImmediateSize::Word)] => 0xF4,
        }

        Pei ("pei") {
            [Direct(None)] => 0xD4,
        }

        Per ("per") {
            [Rel16] => 0x62,
        }

        Pha ("pha") {
            [] => 0x48,
        }
        Phx ("phx") {
            [] => 0xDA,
        }

        Phy ("phy") {
            [] => 0x5A
        }

        Phb ("phb") {
            [] => 0x8B
        }

        Phd ("phd") {
            [] => 0x0B
        }

        Phk ("phk") {
            [] => 0x4B
        }

        Php ("php") {
            [] => 0x08
        }

        Psh ("psh") {
            ![RegA] => Pha [],
            ![RegX] => Phx [],
            ![RegY] => Phy [],
        }

        Pla ("pla") {
            [] => 0x68,
        }
        Plx ("plx") {
            [] => 0xFA,
        }
        Ply ("ply") {
            [] => 0x7A,
        }

        Plb ("plb") {
            [] => 0xAB
        }
        Pld ("pld") {
            [] => 0x2B
        }
        Plp ("plp") {
            [] => 0x28
        }

        Pll ("pll") {
            ![RegA] => Pla [],
            ![RegX] => Plx [],
            ![RegY] => Ply [],
        }

        Stp ("stp") {
            [] => 0xDB
        }
        Wai ("wai") {
            [] => 0xCB
        }
        Tax ("tax") {
            [] => 0xAA
        }
        Tay ("tay") {
            [] => 0xA8
        }
        Tsx ("tsx") {
            [] => 0xBA,
        }
        Txa ("txa") {
            [] => 0x8A,
        }
        Txs ("txs") {
            [] => 0x9A,
        }
        Txy ("txy") {
            [] => 0x9B
        }
        Tya ("tya") {
            [] => 0x98,
        }
        Tyx ("tyx") {
            [] => 0xBB
        }

        Tcd ("tcd") {
            [] => 0x5B
        }

        Tcs ("tcs") {
            [] => 0x1B
        }

        Tdc ("tdc") {
            [] => 0x7B
        }

        Tsc ("tsc") {
            [] => 0x3B
        }

        Tr ("tr") {
            ![RegA, RegX] => Tax [],
            ![RegA, RegY] => Tay [],
            ![RegA, RegS] => Tcs [],
            ![RegX, RegS] => Txs [],
            ![RegS, RegX] => Tsx [],
            ![RegX, RegY] => Txy [],
            ![RegY, RegX] => Tyx [],
        }

        Xba ("xba") {
            [] => 0xEB
        }

        Xce ("xce") {
            [] => 0xFB
        }
    }
}

type W65Opcode = M65Opcode<{ M65Kind::W65 }>;

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

fn cast_compiler<const Kind: M65Kind>(mach: &M65Machine<Kind>) -> Option<CompilerWrapper<'_, M65Machine<Kind>>> {
    match Kind {
        M65Kind::M6502 => None,
        M65Kind::W65 => match CompilerWrapper::from_spec(unsafe{&*(mach as *const _ as *const M65Machine<{M65Kind::W65}>) }) {
            Some(spec) => Some(unsafe{ core::mem::transmute(spec)}),
            None => unreachable!()
        },
    }
}

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
    fn as_compiler(&self) -> Option<CompilerWrapper<'_, Self>> {
        cast_compiler(self)
    }
}


impl W65 {
    pub fn into_opr_pair(op: XvaConst) -> (Operand, Operand) {
        match op {
            xva::XvaConst::Bits(v) => {
                (Operand::Immediate((v & 0xFFFF) as u128), Operand::Immediate((v >> 16) as u128))
            },
            xva::XvaConst::Global(sym, disp) => {
                let sym = RelocSym {sym, kind: AddressKind::Default};
                let disp = NonZero::new(disp);
                (Operand::AbsSymbol(sym, disp), Operand::AbsSymbolSpan(sym, disp, (16..24).into()))
            }
            xva::XvaConst::Label(sym) => {
                let sym = RelocSym {sym, kind: AddressKind::Default};

                (Operand::AbsSymbol(sym, None), Operand::AbsSymbolSpan(sym, None, (16..24).into()))
            }   
        }
    }
}

impl CompilerSpec for M65Machine<{M65Kind::W65}> {
    type Machine = Self;

    fn available_registers(
        &self,
        context: &crate::compiler::CompilerContext,
        mode: Self::MachineMode,
        cat: XvaCategory,
        size: u32,
    ) -> Option<&[Register]> {
        todo!()
    }

    fn promote_size(
        &self,
        context: &crate::compiler::CompilerContext,
        mode: Self::MachineMode,
        cat: XvaCategory,
        size: u32,
    ) -> Option<u32> {
        todo!()
    }

    fn lower_mce(
        &self,
        stmt: &mut crate::xva::XvaStatement,
        mode: Self::MachineMode,
        context: &crate::compiler::CompilerContext,
        features: &crate::mach::FeatureSet,
    ) {
        
        match stmt {
            crate::xva::XvaStatement::Expr(expr) => {
                let mut stmts = Vec::new();

                let dest = Self::areg(expr.dest);

                match expr.op {
                    xva::XvaOpcode::ZeroInit => {
                        match dest {
                            M65Register::Ab | M65Register::Aw => Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![Operand::Immediate(0)])),
                            M65Register::Xb | M65Register::Xw => Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Ldx, vec![Operand::Immediate(0)])),
                            M65Register::Yb | M65Register::Yw => Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Ldy, vec![Operand::Immediate(0)])),
                            M65Register::R(_) => {
                                let addr = dest.to_addr().unwrap();
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Stz, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Stz, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: Address {disp: Some(nzlit!(2)), ..addr}, 
                                })]));
                            }
                            M65Register::Rw(_) => {
                                let addr = dest.to_addr().unwrap();
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Stz, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr, 
                                })]));
                            }
                            _ => panic!("Cannot use {} as a register", dest.name())
                        }
                    },
                    xva::XvaOpcode::Const(xva_const) => {
                        let (oplo, ophi) = Self::into_opr_pair(xva_const);

                        match dest {
                            M65Register::R(_) => {
                                let addr = dest.to_addr().unwrap();
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![oplo]));
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![ophi]));
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: Address {disp: Some(nzlit!(2)), ..addr}, 
                                })]));
                            }

                            M65Register::Rw(_) => {
                                let addr = dest.to_addr().unwrap();
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![oplo]));
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr, 
                                })]));
                            }
                            M65Register::Aw | M65Register::Ab => {
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![oplo]));
                            }
                            M65Register::Xw | M65Register::Xb => {
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Ldx, vec![oplo]));
                            }
                            M65Register::Yw | M65Register::Yb => {
                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![oplo]));
                            }
                            _ => panic!("Cannot load to Register")
                        }
                    },
                    xva::XvaOpcode::Uninit => {},
                    xva::XvaOpcode::Move(src) => {
                        match (dest, Self::areg(src)) {
                            (M65Register::R(_), src @ M65Register::R(_)) => {
                                let dest = dest.to_addr().unwrap();
                                let src = src.to_addr().unwrap();

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: src, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: dest, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: Address {disp: Some(nzlit!(2)), ..src}, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: Address {disp: Some(nzlit!(2)), ..dest}, 
                                })]));
                            }
                            _ => todo!()
                        }
                    },
                    xva::XvaOpcode::ComputeAddr { base, size, index } => todo!(),
                    xva::XvaOpcode::GetFrameAddr(_) => todo!(),
                    xva::XvaOpcode::BinaryOp { op, left, right } => {
                        let op = match op {
                            xva::BinaryOp::Add => {
                                Self::push_instruction(&mut stmts, Instruction::new_nullary(W65Opcode::Clc));
                                W65Opcode::Adc
                            },
                            xva::BinaryOp::Sub => {
                                Self::push_instruction(&mut stmts, Instruction::new_nullary(W65Opcode::Sec));
                                W65Opcode::Sbc
                            },
                            xva::BinaryOp::And =>  W65Opcode::And,
                            xva::BinaryOp::Or =>  W65Opcode::Ora,
                            xva::BinaryOp::Xor =>  W65Opcode::Eor,
                            xva::BinaryOp::ShiftLeft(shift_behaviour) => todo!(),
                            xva::BinaryOp::ShiftRight(shift_behaviour, right_shift_mode) => todo!(),
                        };

                        let src1 = Self::areg(left);
                        
                        let (oplo, ophi) = match right {
                            xva::XvaOperand::Register(reg) => todo!(),
                            xva::XvaOperand::Const(xva_const) => {
                                match xva_const {
                                    xva::XvaConst::Bits(val) => {
                                        (Operand::Immediate((val & 0xFFFF) as u128), Operand::Immediate((val >> 16) as u128))
                                    },
                                    _ => todo!()
                                }
                            },
                            xva::XvaOperand::FrameAddr(_) => todo!(),
                        };

                        match (dest, src1) {
                            (M65Register::R(_), M65Register::R(_)) => {
                                let dest = dest.to_addr().unwrap();
                                let src = src1.to_addr().unwrap();

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: src, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Adc, vec![oplo]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: dest, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Lda, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: Address {disp: Some(nzlit!(2)), ..src}, 
                                })]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Adc, vec![ophi]));

                                Self::push_instruction(&mut stmts, Instruction::new(W65Opcode::Sta, vec![Operand::Memory(MemoryOperand { 
                                    value_size: Some(2), 
                                    addr: Address {disp: Some(nzlit!(2)), ..dest}, 
                                })]));
                            }
                            _ => todo!()
                        }
                    },
                    xva::XvaOpcode::CheckedBinaryOp { op, mode, left, right } => todo!(),
                    xva::XvaOpcode::UnaryOp { op, left } => todo!(),
                    xva::XvaOpcode::Read(xva_operand) => todo!(),
                    xva::XvaOpcode::UMul { left, right } => todo!(),
                    xva::XvaOpcode::SMul { left, right } => todo!(),
                }

                *stmt = xva::XvaStatement::Elaborated(stmts);
            },
            crate::xva::XvaStatement::Write(xva_operand, xva_type, xva_register) => todo!(),
            crate::xva::XvaStatement::Jump(symbol) => {
                *stmt = xva::XvaStatement::RawInstr(Instruction::new(W65Opcode::Bra, vec![Operand::RelSymbol(RelocSym { sym: *symbol, kind: context.local_address_kind }, None)]))
            },
            crate::xva::XvaStatement::Tailcall { dest, params } => todo!(),
            crate::xva::XvaStatement::Call { dest, .. } => {
                match dest {
                    xva::XvaOperand::Register(xva_register) => todo!(),
                    xva::XvaOperand::Const(xva_const) => {
                        let op = xva_const.to_direct_abs(context.local_address_kind, context.global_call_address_kind);
                        *stmt = xva::XvaStatement::RawInstr(Instruction::new(W65Opcode::Jsl, vec![op]));
                    },
                    xva::XvaOperand::FrameAddr(_) => todo!(),
                }
            },
            crate::xva::XvaStatement::Return => {
                *stmt = xva::XvaStatement::RawInstr(Instruction::new_nullary(W65Opcode::Rtl));
            },
            crate::xva::XvaStatement::Trap(_) => {
                *stmt = xva::XvaStatement::RawInstr(Instruction::new(W65Opcode::Brk, vec![Operand::Immediate(0)]));
            },
            
            
            crate::xva::XvaStatement::Elaborated(xva_statements) => unreachable!(),
            crate::xva::XvaStatement::RawInstr(_) |
            crate::xva::XvaStatement::OptGate(_, _) |
            crate::xva::XvaStatement::EndOptGate(_) |
            xva::XvaStatement::Noop(_) |
            crate::xva::XvaStatement::Use(_, _) |
            crate::xva::XvaStatement::Fallthrough(_) => {
                *stmt = xva::XvaStatement::Elaborated(vec![]);
                return
            },
            crate::xva::XvaStatement::Breakpoint => {
                *stmt = xva::XvaStatement::RawInstr(Instruction::new(W65Opcode::Wdm, vec![Operand::Immediate(0)]));
            },
        }
    }

    fn lower_epilogue(
        &self,
        frame: &crate::xva::XvaFrameProperties,
        mode: Self::MachineMode,
    ) -> Vec<crate::xva::XvaStatement> {
        if frame.has_prologue {
            let mut instrs = Vec::new();
            let mut real_frame_size = frame.frame_size & (frame.frame_align - 1);
            if frame.frame_align > 4 {
                todo!()
            } else if frame.frame_align > 1 {
                real_frame_size += 1;
            }

            while real_frame_size >= 2 {
                Self::push_instruction(&mut instrs, Instruction::new_nullary(W65Opcode::Plx));
                real_frame_size -= 2;
            }

            if real_frame_size == 1 {
                Self::push_instruction(&mut instrs, Instruction::new_nullary(W65Opcode::Plb));
            }
            instrs
        } else {
            Vec::new()
        }
    }

    fn emit_prologue(
        &self,
        frame: &mut crate::xva::XvaFrameProperties,
        mode: Self::MachineMode,
    ) -> Vec<crate::instr::Instruction> {
        frame.has_prologue = false;
        let mut instrs = Vec::new();
        let mut real_frame_size = frame.frame_size & (frame.frame_align - 1);
        if frame.frame_align > 4 {
            frame.has_prologue = true;
            todo!()
        } else if frame.frame_align > 1 {
            real_frame_size += 1;
        }

        if real_frame_size > 0 {
            frame.has_prologue = true;
            while real_frame_size >= 2 {
                instrs.push(Instruction::new_nullary(W65Opcode::Pha));
                real_frame_size -= 2;
            }

            if real_frame_size == 1 {
                instrs.push(Instruction::new_nullary(W65Opcode::Phb))
            }

        }

        instrs
    }
}

pub type W65 = M65Machine::<{M65Kind::W65}>;