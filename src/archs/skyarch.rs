//! LC Skyarch ISA
use core::range::{RangeInclusive, RangeInclusiveIter};

use bitflags::bitflags_match;

use crate::{AsRawId, instr::{Address, AddressKind, Instruction, Operand, RegisterKind, RelocSym}, mach::{FeatureSet, MachineSpec, ONE_MACHINE, OneMachine, Opcode, Register, RegisterSpec, Regset, TargetFeatureSpec}, traits::{AsId, BitfieldEncodable, IdType, IntoId, Name}};

#[cfg(feature = "xva")]
use crate::{compiler::{CompilerSpec, CompilerContext}, xva::{XvaCategory, BinaryOp, RightShiftMode, XvaOperand, XvaRegister, XvaStatement}};

pub type SkyarchMachine = OneMachine;

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum SkyarchCoprocInner{
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7
}

impl core::fmt::Debug for SkyarchCoprocInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self as u8).fmt(f)
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct SkyarchCoprocessor(SkyarchCoprocInner);

impl const BitfieldEncodable for SkyarchCoprocessor {
    const MAX_WIDTH: u32 = 3;

    fn decode_bits(val: u128, w: u32) -> Self {
        unsafe { SkyarchCoprocessor::new_unchecked((val & 7) as u8)}
    }

    fn encode_bits(&self) -> u128 {
        self.get() as u128
    }
}

impl SkyarchCoprocessor {
    pub const NUM_COPROC: u8 = 8;
    pub const unsafe fn new_unchecked(val: u8) -> Self {
        unsafe { core::hint::assert_unchecked(val < Self::NUM_COPROC); }
        unsafe { core::mem::transmute(val)}
    }

    pub const fn new(val: u8) -> Option<Self> {
        if val < Self::NUM_COPROC {
            Some(unsafe{Self::new_unchecked(val)})
        } else {
            None
        }
    }

    pub const fn get(&self) -> u8 {
        let val = self.0;

        val as u8
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum SkyarchRegnoInner{
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _20,
    _21,
    _22,
    _23,
    _24,
    _25,
    _26,
    _27,
    _30,
    _31,
    _32,
    _33,
    _34,
    _35,
    _36,
    _37,
}

impl core::fmt::Debug for SkyarchRegnoInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self as u8).fmt(f)
    }
}


#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct SkyarchRegno(SkyarchRegnoInner);

impl const BitfieldEncodable for SkyarchRegno {
    const MAX_WIDTH: u32 = 5;

    fn encode_bits(&self) -> u128 {
        self.get() as u128
    }

    fn decode_bits(val: u128, _: u32) -> Self {
        unsafe { SkyarchRegno::new_unchecked((val & 31) as u8) }
    }
}

impl SkyarchRegno {
    pub const NUM_REGISTERS: u8 = 32;
    pub const unsafe fn new_unchecked(val: u8) -> Self {
        unsafe { core::hint::assert_unchecked(val < Self::NUM_REGISTERS); }
        unsafe { core::mem::transmute(val)}
    }

    pub const fn new(val: u8) -> Option<Self> {
        if val < Self::NUM_REGISTERS {
            Some(unsafe{Self::new_unchecked(val)})
        } else {
            None
        }
    }

    pub const fn get(&self) -> u8 {
        let val = self.0;

        val as u8
    }

    pub const fn gpr(&self) -> SkyarchRegister {
        SkyarchRegister(self.get() as u64)
    }
}


#[macro_export]
macro_rules! skyarch_regno {
    ($e:expr) => {
        const {
            let val: ::core::primitive::u8 = $e;
            assert!(val < $crate::archs::skyarch::SkyarchRegno::NUM_REGISTERS);

            unsafe { $crate::archs::skyarch::SkyarchRegno::new_unchecked(val) }
        }
    }
}

#[allow(non_upper_case_globals)]
impl SkyarchRegno {
    pub const r0: SkyarchRegno = skyarch_regno!(0);
    pub const r15: SkyarchRegno = skyarch_regno!(15);
    pub const r30: SkyarchRegno = skyarch_regno!(30);
    pub const r31: SkyarchRegno = skyarch_regno!(31);
}

pub struct Skyarch;

pub struct SkyarchRegRangeIter{
    range: RangeInclusiveIter<u8>,
}

impl SkyarchRegRangeIter {
    pub fn from_range(range: RangeInclusive<SkyarchRegno>) -> Self {
        let range = RangeInclusive{start: range.start.get(), last: range.last.get()};

        Self{range: range.into_iter()}
    }
}

impl Iterator for SkyarchRegRangeIter {
    type Item = SkyarchRegno;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next()
            .map(|v| unsafe { SkyarchRegno::new_unchecked(v)})
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    fn advance_by(&mut self, n: usize) -> Result<(), std::num::NonZero<usize>> {
        self.range.advance_by(n)
    }
}

#[derive(AsRawId, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SkyarchRegister(pub u64);

#[allow(non_upper_case_globals)]
impl SkyarchRegister {
    pub const r0: SkyarchRegister = SkyarchRegister(0);
    pub const r15: SkyarchRegister = SkyarchRegister(15);
    pub const r31: SkyarchRegister = SkyarchRegister(31);
}

pub const REGISTERS: [SkyarchRegister; 16 * 32] = core::array::from_fn(const |n| SkyarchRegister(n as u64));

impl const AsId<Register> for SkyarchRegister {}

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SkyarchByteSize {
    Byte,
    Half,
    Word,
    #[doc(hidden)]
    __ReservedDouble,
}

impl const BitfieldEncodable for SkyarchByteSize {
    const MAX_WIDTH: u32 = 2;
    
    fn encode_bits(&self) -> u128 {
        *self as u128
    }
    
    fn decode_bits(val: u128, _: u32) -> Self {
        match val {
            0 => Self::Byte,
            1 => Self::Half,
            2 => Self::Word,
            _ => Self::__ReservedDouble,
        }
    }
}

impl core::fmt::Display for SkyarchByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkyarchByteSize::Byte => f.write_str("byte"),
            SkyarchByteSize::Half => f.write_str("half"),
            SkyarchByteSize::Word => f.write_str("word"),
            SkyarchByteSize::__ReservedDouble => f.write_str("8"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Map {
    GeneralPurpose,
    SystemControl,
    Io,
    SystemInfo,
    CoprocessorControl,
    #[doc(hidden)]
    __UnusedMap5,
    #[doc(hidden)]
    __UnusedMap6,
    #[doc(hidden)]
    __UnusedMap7,
    Coprocessor(SkyarchCoprocessor)
}

impl const BitfieldEncodable for Map {
    const MAX_WIDTH: u32 = 4;

    fn encode_bits(&self) -> u128 {
        self.mapno() as u128
    }

    fn decode_bits(val: u128, w: u32) -> Self {
        Self::from_mapno(val as u8)
    }
}

impl Map {
    pub const fn from_mapno(val: u8) -> Self {
         match val {
            0 => Map::GeneralPurpose,
            1 => Map::SystemControl,
            2 => Map::Io,
            3 => Map::SystemInfo,
            4 => Map::CoprocessorControl,
            5 => Map::__UnusedMap5,
            6 => Map::__UnusedMap6,
            7 => Map::__UnusedMap7,
            map @ 8..16 => Map::Coprocessor(unsafe { SkyarchCoprocessor::new_unchecked(map - 8) }),
            _ => panic!("Unknown register number")
        }
    }

    pub const fn mapno(&self) -> u8 {
        match *self {
            Map::GeneralPurpose => 0,
            Map::SystemControl => 1,
            Map::Io => 2,
            Map::SystemInfo => 3,
            Map::CoprocessorControl => 4,
            Map::Coprocessor(coproc) => coproc.get()+8,
            Map::__UnusedMap5 => 5,
            Map::__UnusedMap6 => 6,
            Map::__UnusedMap7 => 7,
        }
    }

    pub const fn reg_in(&self, reg: SkyarchRegno) -> SkyarchRegister {
        let mapno = self.mapno() as u64;
        let regno = reg.get() as u64;

        SkyarchRegister((mapno << 5) | regno)
    }
}

impl core::fmt::Display for SkyarchRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Name for SkyarchRegister {
    fn name(&self) -> &'static str {
        let regno = self.regno().get();
        match self.map() {
            Map::GeneralPurpose => regno_to_static_name!(regno => "r"),
            Map::SystemControl => match regno {
                0 => "sysctl",
                2 => "inttab",
                31 => "intret",
                _ => regno_to_static_name!(regno => "sys"),
            },
            Map::Io => regno_to_static_name!(regno => "io"),
            Map::SystemInfo => regno_to_static_name!(regno => "info"),
            Map::CoprocessorControl => match regno {
                30 => "coe",
                31 => "cop",
                _ => regno_to_static_name!(regno => "cctl")
            },
            Map::Coprocessor(n) => match n.0 {
                SkyarchCoprocInner::_0 => regno_to_static_name!(regno => "c0r"),
                SkyarchCoprocInner::_1 => regno_to_static_name!(regno => "c1r"),
                SkyarchCoprocInner::_2 => regno_to_static_name!(regno => "c2r"),
                SkyarchCoprocInner::_3 => regno_to_static_name!(regno => "c3r"),
                SkyarchCoprocInner::_4 => regno_to_static_name!(regno => "c4r"),
                SkyarchCoprocInner::_5 => regno_to_static_name!(regno => "c5r"),
                SkyarchCoprocInner::_6 => regno_to_static_name!(regno => "c6r"),
                SkyarchCoprocInner::_7 => regno_to_static_name!(regno => "c7r"),
            },
            _ => "**UNKNOWN REGISTER**"
        }
    }
}

impl RegisterSpec for SkyarchRegister {
    
    type MachineMode = SkyarchMachine;

    fn kind(&self) -> RegisterKind {
        match self.map() {
            Map::GeneralPurpose => RegisterKind::GeneralPurpose,
            Map::SystemControl => RegisterKind::System,
            Map::Io => RegisterKind::Special,
            Map::SystemInfo => RegisterKind::System,
            Map::CoprocessorControl => RegisterKind::System,
            Map::Coprocessor(_) => RegisterKind::Special,
            _ => RegisterKind::System
        }
    }

    fn supported_registers(features: &FeatureSet, _: Self::MachineMode) -> crate::mach::Regset {
        Regset::from_registers((1..32).map(SkyarchRegister).chain((0..32).map(|v| SkyarchRegister((2 << 5) | v))))
    }

    fn size(&self, _: Self::MachineMode) -> u32 {
        4
    }

    #[cfg(feature = "xva")]
    fn category(&self, _: Self::MachineMode) -> crate::xva::XvaCategory {
        match self.map() {
            Map::GeneralPurpose => XvaCategory::Int,
            _ => XvaCategory::Custom(self.kind())
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        self == other
    }

    fn from_bit(bit: u32, _: Self::MachineMode) -> Option<Self> {
        let upper = bit >> 5;
        let regno = (bit & 31) as u64;

        match upper {
            0 => Some(Self(regno)),
            1 => Some(Self(regno | 0x40)),
            n @ 2..10 => Some(Self(regno | (0x100 + ((n as u64) << 5)))),
            _ => None,
        }
    }

    fn regmap_bit(self) -> Option<u32> {
        let regno = self.regno().get() as u32;
        match self.map() {
            Map::GeneralPurpose => Some(regno),
            Map::Io => Some(regno | (1 << 5)),
            Map::Coprocessor(n) => Some(regno | (((n.get() as u32) + 2) << 5)),
            _ => None,
        }
    }
}

impl SkyarchRegister {
    pub const fn regno(&self) -> SkyarchRegno {
        unsafe { SkyarchRegno::new_unchecked((self.0 & 31) as u8) }
    }

    pub const fn map(&self) -> Map {
       Map::from_mapno(((self.0 >> 5) & 15) as u8)
    }
}

#[derive(AsRawId, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SkyarchOpcode(u64);

impl const AsId<Opcode> for SkyarchOpcode{}


macro_rules! skyarch_opcodes {
    (#is_synthetic true) => {
        true
    };
    (#is_synthetic) => {
        false
    };
    ($vis:vis impl $name:ident (enum $decoded:ident) {
        $($(#[$vmeta:meta])* $(!$(@ $_synthetic:tt)?)? $instr:ident $mnemonic:literal $({$(!$(#[$pmeta:meta])* $primary:ident $(@ $pwidth:literal)?: $pty:ty,)? $($(#[$fmeta:meta])* $fname:ident @ $base:literal$(..$($end:literal)?)?: $ty:ty),* $(,)?})? = $opcode:literal $(..$encoding_end:literal)?),+ $(,)?
    }) => {
        impl Name for $name {
            fn name(&self) -> &'static str {
                self.decode().name()
            }
        }

        impl Name for $decoded {
            fn name(&self) -> &'static str {
                match self {
                    $(Self:: $instr {..} => $mnemonic),*
                }
            }
        }

        impl $name {
            pub const fn decode(&self) -> $decoded {
                match (self.0 & 0xFF, (self.0 & 0x10000000) != 0) {
                    $(($opcode $(.. $encoding_end)?, skyarch_opcodes!(#is_synthetic $(true $(@ $_synthetic:tt)?)?)) => {
                        $(
                            $(
                                let $primary = {
                                    let width = const {
                                        let width = ($($pwidth,)? <$pty as BitfieldEncodable>::MAX_WIDTH,).0;

                                        assert!(width <= <$pty as BitfieldEncodable>::MAX_WIDTH, 
                                            ::core::concat!("Width of primary ", ::core::stringify!($primary), " (", $(::core::stringify!($pwidth),)? ") exceeds width of type")
                                        );

                                        width
                                    };

                                    <$pty as BitfieldEncodable>::decode_bits((self.0 & ((1 << width) - 1)) as u128, width)
                                };
                            )?

                            $(
                                let $fname = {
                                    let (base, width) = const {
                                        let base: u32 = $base;
                                        let end: u32 = ($($($end,)? base +<$ty as BitfieldEncodable>::MAX_WIDTH,)? base + 1,).0;
                                        let width = end - base;


                                        assert!(width <= <$ty as BitfieldEncodable>::MAX_WIDTH, 
                                                ::core::concat!("Range of field", ::core::stringify!($field), " (", ::core::stringify!($base), $("..", $(::core::stringify!($end),)?)? ") exceeds width of type")
                                            );

                                        assert!(end <= 24, ::core::concat!("Range of field", ::core::stringify!($field), " (", ::core::stringify!($base), $("..", $(::core::stringify!($end),)?)? ") exceeds bounds of the payload field"));

                                        (8 + base, width)
                                    };

                                    <$ty as BitfieldEncodable>::decode_bits(((self.0 >> base) & ((1 << width) - 1)) as u128, width)
                                };
                            )*
                        )?

                        $decoded :: $instr $({$($primary,)? $($fname),*})?
                    },)*
                    _ => $decoded :: InvalidEncoding,
                }
            }
        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub enum $decoded {
            $($instr $({$($(#[$pmeta])* $primary: $pty,)? $($(#[$fmeta])* $fname: $ty),*})?),*
        }

        impl $decoded {
            pub const fn encode(&self) -> $name {
                match self {
                    $(Self::$instr $({$($primary,)? $($fname),*})? => {
                        const OPCODE: u8 = $opcode;
                        let val: u32 = OPCODE as u32 $($(| {
                            let width = const {
                                let width = ($($pwidth,)? <$pty as BitfieldEncodable>::MAX_WIDTH,).0;

                                assert!(width <= <$pty as BitfieldEncodable>::MAX_WIDTH, 
                                    ::core::concat!("Width of primary ", ::core::stringify!($primary), " (", $(::core::stringify!($pwidth),)? ") exceeds width of type")
                                );

                                width
                            };

                            let val = (<$pty as BitfieldEncodable>::encode_bits($primary) as u32) & 1u32.unbounded_shl(width).wrapping_sub(1);

                            val
                        })? $(| {
                            let (base, width) = const {
                                let base: u32 = $base;
                                let end: u32 = ($($($end,)? base +<$ty as BitfieldEncodable>::MAX_WIDTH,)? base + 1,).0;
                                let width = end - base;


                                assert!(width <= <$ty as BitfieldEncodable>::MAX_WIDTH, 
                                        ::core::concat!("Range of field", ::core::stringify!($field), " (", ::core::stringify!($base), $("..", $(::core::stringify!($end),)?)? ") exceeds width of type")
                                    );

                                assert!(end <= 24, ::core::concat!("Range of field", ::core::stringify!($field), " (", ::core::stringify!($base), $("..", $(::core::stringify!($end),)?)? ") exceeds bounds of the payload field"));

                                (8 + base, width)
                            };

                            let val = (<$ty as BitfieldEncodable>::encode_bits($fname) as u32) & 1u32.unbounded_shl(width).wrapping_sub(1);

                            val << base
                        })*)?;

                        $name ((val as u64) $(| 0x10000000 $(@@ $_synthetic)?)?)
                    })*
                }
            }
        }

        const ALL_OPCODES: [SkyarchOpcode; ${count($instr)}] = [$($name($opcode)),*];
    }
}

impl core::fmt::Display for SkyarchInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkyarchInstruction::Und00 => f.write_str("und"),
            SkyarchInstruction::Pause { k } => f.write_fmt(format_args!("pause {k}")),
            SkyarchInstruction::Mov { dest, ssrc, latency, cond, dir, map } => {
                let (srcmap, destmap) = if *dir {
                    (Map::GeneralPurpose, *map)
                } else {
                    (*map, Map::GeneralPurpose)
                };

                let srcr = srcmap.reg_in(*ssrc);
                let destr = destmap.reg_in(*dest);

                f.write_str("mov")?;

                if *latency {
                    f.write_str("l")?;
                }

                cond.fmt(f)?;

                f.write_str(" ")?;
                f.write_str(destr.name())?;
                f.write_str(", ")?;
                f.write_str(srcr.name())
            },
            SkyarchInstruction::Ld { dest, src, width, mode } => {
                match mode {
                    SkyarchLoadStoreMode::PostInc => f.write_str("pop ")?,
                    SkyarchLoadStoreMode::PreDec => f.write_str("lddec ")?,
                    _ => f.write_str("ld ")?,
                }

                let dest = dest.gpr();
                let src = src.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;
                src.fmt(f)?;

                f.write_str(", ")?;

                width.fmt(f)
            },
            SkyarchInstruction::St { dest, src, width, mode } => {
                match mode {
                    SkyarchLoadStoreMode::PostInc => f.write_str("stinc ")?,
                    SkyarchLoadStoreMode::PreDec => f.write_str("push ")?,
                    _ => f.write_str("st ")?,
                }

                let dest = dest.gpr();
                let src = src.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;
                src.fmt(f)?;

                f.write_str(", ")?;

                width.fmt(f)
            },
            SkyarchInstruction::Ldi { dest, signed, imm } => {
                let dest = dest.gpr();
                f.write_str("ldi ")?;
                dest.fmt(f)?;
                f.write_str(", ")?;
                if *signed  {
                    imm.fmt(f)
                } else {
                    let imm = *imm as u16;
                    imm.fmt(f)
                }
            },
            SkyarchInstruction::Lra { dest, signed, imm } => {
                let dest = dest.gpr();
                f.write_str("lra ")?;
                dest.fmt(f)?;
                f.write_str(", ")?;
                if *signed  {
                    imm.fmt(f)
                } else {
                    let imm = *imm as u16;
                    imm.fmt(f)
                }
            },
            SkyarchInstruction::Xchg { reg1, reg2 } => {
                let reg1 = reg1.gpr();
                let reg2 = reg2.gpr();

                f.write_str("xchg ")?;
                reg1.fmt(f)?;
                f.write_str(", ")?;
                reg2.fmt(f)
            },
            SkyarchInstruction::Addi { dest, signed, supress_flags, higher_half, imm } => {
                let dest = dest.gpr();
                f.write_str("addi")?;
                if *supress_flags {
                    f.write_str("c")?;
                }
                if *higher_half {
                    f.write_str("h")?;
                }
                f.write_str(" ")?;
                dest.fmt(f)?;
                f.write_str(", ")?;
                if *signed  {
                    imm.fmt(f)
                } else {
                    let imm = *imm as u16;
                    imm.fmt(f)
                }
            },
            SkyarchInstruction::Add { dest, src1, src2, supress_flags, shift, shift_polarity } => {
                f.write_str("add")?;
                if *supress_flags {
                    f.write_str("c")?;
                }

                f.write_str(" ")?;
                let (src1q, src2q) = if *shift_polarity {
                    (*shift, 0)
                } else {
                    (0, *shift)
                };

                let dest = dest.gpr();

                let src1 = src1.gpr();
                let src2 = src2.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;

                src1.fmt(f)?;

                if src1q > 0 {
                    f.write_str(" << ")?;
                    src1q.fmt(f)?;
                }
                f.write_str(", ")?;


                src2.fmt(f)?;

                if src2q > 0 {
                    f.write_str(" << ")?;
                    src2q.fmt(f)?;
                }

                Ok(())
            },
            SkyarchInstruction::Sub { dest, src1, src2, supress_flags, shift, shift_polarity } => {
                f.write_str("sub")?;
                if *supress_flags {
                    f.write_str("c")?;
                }

                f.write_str(" ")?;
                let (src1q, src2q) = if *shift_polarity {
                    (*shift, 0)
                } else {
                    (0, *shift)
                };

                let dest = dest.gpr();

                let src1 = src1.gpr();
                let src2 = src2.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;

                src1.fmt(f)?;

                if src1q > 0 {
                    f.write_str(" << ")?;
                    src1q.fmt(f)?;
                }
                f.write_str(", ")?;


                src2.fmt(f)?;

                if src2q > 0 {
                    f.write_str(" << ")?;
                    src2q.fmt(f)?;
                }

                Ok(())
            },
            SkyarchInstruction::And { dest, src1, src2, supress_flags, shift, shift_polarity, invert } => {
                f.write_str("and")?;
                if *supress_flags {
                    f.write_str("c")?;
                }

                let (invert1, invert2) = ((invert & 1) != 0, (invert & 2) != 0);

                f.write_str(" ")?;
                let (src1q, src2q) = if *shift_polarity {
                    (*shift, 0)
                } else {
                    (0, *shift)
                };

                let dest = dest.gpr();

                let src1 = src1.gpr();
                let src2 = src2.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;

                if invert1 {
                    f.write_str("~")?;
                }

                src1.fmt(f)?;

                if src1q > 0 {
                    f.write_str(" << ")?;
                    src1q.fmt(f)?;
                }
                f.write_str(", ")?;

                if invert2 {
                    f.write_str("~")?;
                }

                src2.fmt(f)?;

                if src2q > 0 {
                    f.write_str(" << ")?;
                    src2q.fmt(f)?;
                }

                Ok(())
            },
            SkyarchInstruction::Or { dest, src1, src2, supress_flags, shift, shift_polarity, invert } => {
                f.write_str("or")?;
                if *supress_flags {
                    f.write_str("c")?;
                }

                let (invert1, invert2) = ((invert & 1) != 0, (invert & 2) != 0);

                f.write_str(" ")?;
                let (src1q, src2q) = if *shift_polarity {
                    (*shift, 0)
                } else {
                    (0, *shift)
                };

                let dest = dest.gpr();

                let src1 = src1.gpr();
                let src2 = src2.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;

                if invert1 {
                    f.write_str("~")?;
                }

                src1.fmt(f)?;

                if src1q > 0 {
                    f.write_str(" << ")?;
                    src1q.fmt(f)?;
                }
                f.write_str(", ")?;

                if invert2 {
                    f.write_str("~")?;
                }

                src2.fmt(f)?;

                if src2q > 0 {
                    f.write_str(" << ")?;
                    src2q.fmt(f)?;
                }

                Ok(())
            },
            SkyarchInstruction::Xor { dest, src1, src2, supress_flags, shift, shift_polarity, invert } => {
                f.write_str("xor")?;
                if *supress_flags {
                    f.write_str("c")?;
                }

                let (invert1, invert2) = ((invert & 1) != 0, (invert & 2) != 0);

                f.write_str(" ")?;
                let (src1q, src2q) = if *shift_polarity {
                    (*shift, 0)
                } else {
                    (0, *shift)
                };

                let dest = dest.gpr();

                let src1 = src1.gpr();
                let src2 = src2.gpr();

                dest.fmt(f)?;
                f.write_str(", ")?;

                if invert1 {
                    f.write_str("~")?;
                }

                src1.fmt(f)?;

                if src1q > 0 {
                    f.write_str(" << ")?;
                    src1q.fmt(f)?;
                }
                f.write_str(", ")?;

                if invert2 {
                    f.write_str("~")?;
                }

                src2.fmt(f)?;

                if src2q > 0 {
                    f.write_str(" << ")?;
                    src2q.fmt(f)?;
                }

                Ok(())
            },
            SkyarchInstruction::Fsl { dest, value, quantity, supress_flags, invert_sign, wrap_quantity, remainder } => todo!(),
            SkyarchInstruction::Fsr { dest, value, quantity, supress_flags, invert_sign, wrap_quantity, remainder } => todo!(),
            SkyarchInstruction::Jmp { cond, link, offset } => {
                f.write_str("jmp")?;

                cond.fmt(f)?;

                f.write_str(" ")?;

                let link = link.gpr();

                link.fmt(f)?;
                f.write_str(", ")?;

                let off = *offset << 2;

                off.fmt(f)
            },
            SkyarchInstruction::Jmpr { cond, link, dest } => {
                f.write_str("jmpr")?;

                cond.fmt(f)?;
                f.write_str(" ")?;

                let link = link.gpr();
                let dest = dest.gpr();

                link.fmt(f)?;
                f.write_str(", ")?;
                dest.fmt(f)
            },
            SkyarchInstruction::In { dest, port, width } => {
                f.write_str("in ")?;
                let dest = dest.gpr();

                dest.fmt(f)?;

                f.write_str(", ")?;

                port.fmt(f)?;
                f.write_str(", ")?;
                width.fmt(f)
            },
            SkyarchInstruction::Out { src, port, width } => {
                f.write_str("out ")?;
                let dest = src.gpr();

                dest.fmt(f)?;

                f.write_str(", ")?;

                port.fmt(f)?;
                f.write_str(", ")?;
                width.fmt(f)
            },
            SkyarchInstruction::Ldflags { dest, mask } => {
                f.write_str("ldflags ")?;
                let dest = dest.gpr();

                dest.fmt(f)?;

                f.write_str(", ")?;

                mask.fmt(f)
            },
            SkyarchInstruction::Stflags { src, mask } => {
                f.write_str("stflags ")?;
                let dest = src.gpr();

                dest.fmt(f)?;

                f.write_str(", ")?;

                mask.fmt(f)
            },
            SkyarchInstruction::Xvp => f.write_str("xvp"),
            SkyarchInstruction::Cpi { coproc, func, payload } => {
                let n = coproc.get();
                f.write_str("cpi")?;
                n.fmt(f)?;

                f.write_str(" ")?;

                f.write_fmt(format_args!("{func:#04X}, {payload:#08}"))
            },
            SkyarchInstruction::Ncpi { coproc, func, payload } => {
                let n = coproc.get();
                f.write_str("ncpi")?;
                n.fmt(f)?;

                f.write_str(" ")?;

                f.write_fmt(format_args!("{func:#03X}, {payload:#08}"))
            },
            SkyarchInstruction::CpiEf { coproc, func, payload } => {
                let n = coproc.get();
                f.write_str("cpi")?;
                n.fmt(f)?;

                f.write_str("ef ")?;

                f.write_fmt(format_args!("{func:#04X}, {payload:#08}"))
            },
            SkyarchInstruction::NcpiEf { coproc, func, payload } => {
                let n = coproc.get();
                f.write_str("ncpi")?;
                n.fmt(f)?;

                f.write_str("ef ")?;

                f.write_fmt(format_args!("{func:#04X}, {payload:#08}"))
            },
            SkyarchInstruction::UndFF => f.write_str("und"),
            SkyarchInstruction::InvalidEncoding => f.write_str("und"),
            SkyarchInstruction::LdiW { dest, signed } => {
                let dest = dest.gpr();
                f.write_str("ldi ")?;
                dest.fmt(f)?;
                f.write_str(", ")
            },
            SkyarchInstruction::LraW { dest, signed } => {
                let dest = dest.gpr();
                f.write_str("lra ")?;
                dest.fmt(f)?;
                f.write_str(", ")
            },
            SkyarchInstruction::AddiW { dest, signed, supress_flags, higher_half } => {
                let dest = dest.gpr();
                f.write_str("addi")?;
                if *supress_flags {
                    f.write_str("c")?;
                }
                if *higher_half {
                    f.write_str("h")?;
                }
                f.write_str(" ")?;
                dest.fmt(f)?;
                f.write_str(", ")
            },
            SkyarchInstruction::JmpW { cond, link, dest } => {
                f.write_str("jmpw")?;

                cond.fmt(f)?;


                f.write_str(" ")?;

                let link = link.gpr();
                let dest = dest.gpr();

                link.fmt(f)?;
                f.write_str(", ")?;
                dest.fmt(f)?;
                f.write_str(", ")
            },
        }
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct SkyarchFlags : u8 {
        const CARRY = 0x01;
        const OVERFLOW = 0x02;
        const NEGATIVE = 0x04;
        const ZERO = 0x08;
        const PARITY = 0x10;
    }
}

impl const BitfieldEncodable for SkyarchFlags {
    const MAX_WIDTH: u32 = 5;

    fn decode_bits(val: u128, _: u32) -> Self {
        SkyarchFlags::from_bits_truncate(val as u8)
    }

    fn encode_bits(&self) -> u128 {
        self.bits() as u128
    }
}

impl core::fmt::Display for SkyarchFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bit in self.iter() {
            bitflags_match!(bit, {
                SkyarchFlags::CARRY => f.write_str("c"),
                SkyarchFlags::OVERFLOW => f.write_str("v"),
                SkyarchFlags::NEGATIVE => f.write_str("n"),
                SkyarchFlags::ZERO => f.write_str("z"),
                SkyarchFlags::PARITY => f.write_str("p"),
                _ => Ok(())
            })?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SkyarchConditionCode {
    Never = 0,
    Carry = 1,
    Zero = 2,
    Overflow = 3,
    CarryOrEqual = 4,
    SignedLess = 5,
    SignedLessOrEq = 6,
    Negative = 7,
    Positive = 8,
    SignedGreater = 9,
    SignedGreaterOrEq = 10,
    Above = 11,
    NotOverflow = 12,
    NotZero = 13,
    NotCarry = 14,
    Always = 15,
}

impl core::fmt::Display for SkyarchConditionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SkyarchConditionCode::Never => "s",
            SkyarchConditionCode::Carry => "c",
            SkyarchConditionCode::Zero => "z",
            SkyarchConditionCode::Overflow => "v",
            SkyarchConditionCode::CarryOrEqual => "be",
            SkyarchConditionCode::SignedLess => "le",
            SkyarchConditionCode::SignedLessOrEq => "le",
            SkyarchConditionCode::Negative => "n",
            SkyarchConditionCode::Positive => "ps",
            SkyarchConditionCode::SignedGreater => "gt",
            SkyarchConditionCode::SignedGreaterOrEq => "ge",
            SkyarchConditionCode::Above => "a",
            SkyarchConditionCode::NotOverflow => "nv",
            SkyarchConditionCode::NotZero => "nz",
            SkyarchConditionCode::NotCarry => "nc",
            SkyarchConditionCode::Always => "",
        };

        f.write_str(s)
    }
}

impl const BitfieldEncodable for SkyarchConditionCode {
    const MAX_WIDTH: u32 = 4;
    fn decode_bits(val: u128, w: u32) -> Self {
        let bits = (val & 15) as u8;
        unsafe { core::mem::transmute(bits)}
    }

    fn encode_bits(&self) -> u128 {
        *self as u128
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SkyarchLoadStoreMode {
    Default,
    PostInc,
    #[doc(hidden)]
    _Reserved2,
    PreDec,
}

impl const BitfieldEncodable for SkyarchLoadStoreMode {
    const MAX_WIDTH: u32 = 2;
    fn decode_bits(val: u128, w: u32) -> Self {
        let bits = (val & 3) as u8;
        unsafe { core::mem::transmute(bits)}
    }

    fn encode_bits(&self) -> u128 {
        *self as u128
    }
}


skyarch_opcodes! {
    pub impl SkyarchOpcode (enum SkyarchInstruction) {
        Und00 "und" = 0x00,
        Pause "pause" {k @ 0..6: u8} = 0x01,
        Mov "mov" {dest @ 0..5: SkyarchRegno, ssrc @ 5..10: SkyarchRegno, latency @ 11: bool, cond @ 12..16 : SkyarchConditionCode, dir @ 17: bool, map @ 18..22: Map} = 0x02,
        Ld "ld" {dest @ 0..5: SkyarchRegno, src @ 5..10: SkyarchRegno, width @ 10..12: SkyarchByteSize, mode @ 22..24: SkyarchLoadStoreMode} = 0x03,
        St "st" {dest @ 0..5: SkyarchRegno, src @ 5..10: SkyarchRegno, width @ 10..12: SkyarchByteSize, mode @ 22..24: SkyarchLoadStoreMode} = 0x04,
        Ldi "ldi" {dest @ 0..5: SkyarchRegno, signed @ 5: bool, imm @ 8..: i16} = 0x05,
        Lra "lra" {dest @ 0..5: SkyarchRegno, signed @ 5: bool, imm @ 8..: i16} = 0x06,
        Xchg "xchg" {reg1 @ 0..5: SkyarchRegno, reg2 @ 5..10: SkyarchRegno} = 0x07,
        Addi "addi" {dest @ 0..5: SkyarchRegno, signed @ 5: bool, supress_flags @ 6: bool, higher_half @ 7: bool, imm @ 8..: u16} = 0x08,
        Add "add" {dest @ 0..5: SkyarchRegno, src1 @ 5..10: SkyarchRegno, src2 @ 10..15 : SkyarchRegno, supress_flags @ 16: bool, shift @ 17..22: u32, shift_polarity @ 22: bool} = 0x09,
        Sub "sub" {dest @ 0..5: SkyarchRegno, src1 @ 5..10: SkyarchRegno, src2 @ 10..15 : SkyarchRegno, supress_flags @ 16: bool, shift @ 17..22: u32, shift_polarity @ 22: bool} = 0x0A,
        And "and" {dest @ 0..5: SkyarchRegno, src1 @ 5..10: SkyarchRegno, src2 @ 10..15 : SkyarchRegno, supress_flags @ 16: bool, shift @ 17..22: u32, shift_polarity @ 22: bool, invert @ 23..24: u8} = 0x0B,
        Or "or" {dest @ 0..5: SkyarchRegno, src1 @ 5..10: SkyarchRegno, src2 @ 10..15 : SkyarchRegno, supress_flags @ 16: bool, shift @ 17..22: u32, shift_polarity @ 22: bool, invert @ 23..24: u8} = 0x0C,
        Xor "xor" {dest @ 0..5: SkyarchRegno, src1 @ 5..10: SkyarchRegno, src2 @ 10..15 : SkyarchRegno, supress_flags @ 16: bool, shift @ 17..22: u32, shift_polarity @ 22: bool, invert @ 23..24: u8} = 0x0D,
        Fsl "fsl" {dest @ 0..5: SkyarchRegno, value @ 5..10: SkyarchRegno, quantity @ 10..15: SkyarchRegno, supress_flags @ 15: bool, invert_sign @ 16: bool, wrap_quantity @ 18: bool, remainder @ 19..24: SkyarchRegno} = 0x0E,
        Fsr "fsr" {dest @ 0..5: SkyarchRegno, value @ 5..10: SkyarchRegno, quantity @ 10..15: SkyarchRegno, supress_flags @ 15: bool, invert_sign @ 16: bool, wrap_quantity @ 18: bool, remainder @ 19..24: SkyarchRegno} = 0x0F,
        Jmp "jmp" {cond @ 0..4: SkyarchConditionCode, link @ 4..9: SkyarchRegno, offset @ 9..24: i32} = 0x10,
        Jmpr "jmpr" {cond @ 0..4: SkyarchConditionCode, link @ 4..9: SkyarchRegno, dest @ 9..14: SkyarchRegno} = 0x11,
        In "in" {dest @ 0..5: SkyarchRegno, port @ 5..: u8, width @ 19..24: u8} = 0x14,
        Out "out" {src @ 0..5: SkyarchRegno, port @ 5..: u8, width @ 19..24: u8} = 0x15,
        Ldflags "ldflags" {dest @ 0..5: SkyarchRegno, mask @ 5..10: SkyarchFlags} = 0x18,
        Stflags "stflags" {src @ 0..5: SkyarchRegno, mask @ 5..10: SkyarchFlags} = 0x19,
        Xvp "xvp" = 0x1A,

        Cpi "cpi" {!coproc: SkyarchCoprocessor, func @ 0..4: u8, payload @ 4..20: u32} = 0x20..0x28,
        Ncpi "ncpi" {!coproc: SkyarchCoprocessor, func @ 0..4: u8, payload @ 4..20: u32} = 0x28..0x30,
        CpiEf "cpief" {!coproc: SkyarchCoprocessor, func @ 0..6: u8, payload @ 6..20: u32} = 0x30..0x38,
        NcpiEf "ccpief" {!coproc: SkyarchCoprocessor, func @ 0..6: u8, payload @ 6..20: u32} = 0x38..0x40,


        UndFF "und" = 0xFF,

        !InvalidEncoding "**UNKNOWN INSTRUCTION**" = 0x00,
        !LdiW "ldiw" {dest @ 0..5: SkyarchRegno, signed @ 5: bool} = 0x05,
        !LraW "lraw" {dest @ 0..5: SkyarchRegno, signed @ 5: bool} = 0x06,
        !AddiW "addiw" {dest @ 0..5: SkyarchRegno, signed @ 5: bool, supress_flags @ 6: bool, higher_half @ 7: bool} = 0x08,
        !JmpW "jmpw" {cond @ 0..4: SkyarchConditionCode, link @ 4..9: SkyarchRegno, dest @ 9..14: SkyarchRegno} = 0x10,
    }
}

impl const IntoId<Opcode> for SkyarchInstruction {
    fn into_id(self) -> Opcode {
        Opcode::new(self.encode())
    }
}

def_features! {
    pub enum SkyarchTargetFeature {

    }
}

impl MachineSpec for Skyarch {
    type Opcode = SkyarchOpcode;

    const OPCODES: &[crate::mach::Opcode] = as_id_array!(ALL_OPCODES => Opcode);

    type Register = SkyarchRegister;

    const REGISTERS: &[Register] = as_id_array!(REGISTERS => Register);

    type MachineMode = SkyarchMachine;

    const MACH_MODES: &[crate::mach::MachineMode] = ONE_MACHINE;

    type TargetFeature = SkyarchTargetFeature;

    fn name(&self) -> &'static str {
        "skyarch"
    }


    fn pretty_print_instr(&self, instr: Self::Opcode, _: Self::MachineMode, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use core::fmt::Display as _;
        instr.decode().fmt(f)
    }

    #[cfg(feature = "xva")]
    fn as_compiler(&self) -> Option<&dyn crate::compiler::CheckCompiler<Machine = Self>> {
        Some(self)
    }
}

const GPRS: [SkyarchRegister; 31] = core::array::from_fn(const |v| SkyarchRegister((v as u64) + 1));


#[cfg(feature = "xva")]
impl CompilerSpec for Skyarch {
    type Machine = Self;

    fn available_registers(
        &self,
        context: &crate::compiler::CompilerContext,
        mode: Self::MachineMode,
        cat: XvaCategory,
        size: u32,
    ) -> Option<&[Register]> {
        match cat {
            XvaCategory::Null => Some(&[]),
            XvaCategory::Condition => None,
            XvaCategory::Int|
            XvaCategory::Float|
            XvaCategory::VectorAny|
            XvaCategory::VectorInt|
            XvaCategory::VectorFloat|
            XvaCategory::Aggregate => {
                if size == 4 {
                    Some(as_id_array!(GPRS => Register))
                } else {
                    None
                }
            },
            XvaCategory::Custom(_) => None,
        }
    }

    fn promote_size(
        &self,
        _: &crate::compiler::CompilerContext,
        _: Self::MachineMode,
        _: XvaCategory,
        size: u32,
    ) -> Option<u32> {
        if size < 4 {
            Some(4)
        } else {
            None
        }
    }

    fn lower_mce(&self, stmt: &mut crate::xva::XvaStatement, _: Self::MachineMode, _: &CompilerContext, _: &FeatureSet) {
        let mut preamble = Vec::new();
        match &*stmt {
            crate::xva::XvaStatement::Expr(expr) => {
                let dest = Self::areg(expr.dest);
                let instr = match expr.op {
                    crate::xva::XvaOpcode::Uninit => {
                        *stmt = XvaStatement::Elaborated(vec![]);
                        return;
                    }
                    crate::xva::XvaOpcode::ZeroInit => {
                        Instruction::new_nullary(SkyarchInstruction::Mov { dest: dest.regno(), ssrc: SkyarchRegno::r0, latency: false, cond: SkyarchConditionCode::Always, dir: false, map: Map::GeneralPurpose})
                    },
                    crate::xva::XvaOpcode::Const(xva_const) => {
                        match xva_const {
                            crate::xva::XvaConst::Bits(v) => {
                                Instruction::new(SkyarchInstruction::LdiW { dest: dest.regno(), signed: false }, vec![Operand::Immediate(v as u128)])
                            },
                            xva_const => {
                                let addr = xva_const.to_direct_rel(crate::instr::AddressKind::Default, crate::instr::AddressKind::Default);
                                Instruction::new(SkyarchInstruction::LraW { dest: dest.regno(), signed: false }, vec![addr])
                            }
                        }
                    },
                    crate::xva::XvaOpcode::Move(src) => {
                        let src = Self::areg(src);
                        Instruction::new_nullary(SkyarchInstruction::Mov { dest: dest.regno(), ssrc: src.regno(), latency: false, cond: SkyarchConditionCode::Always, dir: false, map: Map::GeneralPurpose})
                    },
                    crate::xva::XvaOpcode::ComputeAddr { base, size, index } => todo!(),
                    crate::xva::XvaOpcode::GetFrameAddr(_) => todo!(),
                    crate::xva::XvaOpcode::BinaryOp { op, left, right } => {
                        'a: {
                            let src1 = Self::areg(left);
                            let src2 = match right {
                                crate::xva::XvaOperand::Register(src) => Self::areg(src),
                                crate::xva::XvaOperand::Const(xva_const) => {
                                    let opr = xva_const.to_readable(AddressKind::Default, AddressKind::Default, true, None);
                                    let (vval, signext) = match xva_const {
                                        crate::xva::XvaConst::Bits(v) => {
                                            if v < u16::MAX as u64 {
                                                (Some(v as u16), false)
                                            } else if v > (i16::MIN as u64) {
                                                (Some(v as u16), true)
                                            } else {
                                                (None, false)
                                            }
                                        },
                                        _ => todo!(),
                                    };

                                    let instr = match (op, vval) {
                                        (BinaryOp::Add, Some(vval)) => {
                                            break 'a Instruction::new_nullary(SkyarchInstruction::Addi { dest: dest.regno(), signed: signext, supress_flags: true, higher_half: false, imm: vval })
                                        }
                                        (BinaryOp::Add, None) => {
                                            break 'a Instruction::new(SkyarchInstruction::AddiW { dest: dest.regno(), signed: signext, supress_flags: true, higher_half: false }, vec![opr])
                                        }
                                        (_, Some(vval)) => {
                                            Instruction::new_nullary(SkyarchInstruction::Ldi { dest: SkyarchRegno::r15, signed: signext, imm: vval as i16 })
                                        }
                                        (_, None) => {
                                            Instruction::new(SkyarchInstruction::LdiW { dest: SkyarchRegno::r15, signed: signext }, vec![opr])
                                        }
                                    };

                                    preamble.push(XvaStatement::RawInstr(instr));
                                    SkyarchRegister::r15
                                },
                                crate::xva::XvaOperand::FrameAddr(_) => todo!(),
                            };



                            let instr = match op {
                                crate::xva::BinaryOp::Add => SkyarchInstruction::Add { dest: dest.regno(), src1: src1.regno(), src2: src2.regno(), supress_flags: true, shift: 0, shift_polarity: false },
                                crate::xva::BinaryOp::Sub => SkyarchInstruction::Sub { dest: dest.regno(), src1: src1.regno(), src2: src2.regno(), supress_flags: true, shift: 0, shift_polarity: false },
                                crate::xva::BinaryOp::And => SkyarchInstruction::And { dest: dest.regno(), src1: src1.regno(), src2: src2.regno(), supress_flags: true, shift: 0, shift_polarity: false, invert: 0 },
                                crate::xva::BinaryOp::Or => SkyarchInstruction::Or { dest: dest.regno(), src1: src1.regno(), src2: src2.regno(), supress_flags: true, shift: 0, shift_polarity: false, invert: 0 },
                                crate::xva::BinaryOp::Xor => SkyarchInstruction::Xor { dest: dest.regno(), src1: src1.regno(), src2: src2.regno(), supress_flags: true, shift: 0, shift_polarity: false, invert: 0 },
                                crate::xva::BinaryOp::ShiftLeft(shift_behaviour) => SkyarchInstruction::Fsl { dest: dest.regno(), value: src1.regno(), quantity: src2.regno(), supress_flags: true, invert_sign: false, wrap_quantity: matches!(shift_behaviour, crate::xva::ShiftBehaviour::WrapQuantity), remainder: SkyarchRegno::r0 },
                                crate::xva::BinaryOp::ShiftRight(shift_behaviour ,mode) => SkyarchInstruction::Fsr { dest: dest.regno(), value: src1.regno(), quantity: src2.regno(), supress_flags: true, invert_sign: matches!(mode, RightShiftMode::Signed), wrap_quantity: matches!(shift_behaviour, crate::xva::ShiftBehaviour::WrapQuantity), remainder: SkyarchRegno::r0 },
                            };

                            Instruction::new_nullary(instr)
                        }
                    },
                    crate::xva::XvaOpcode::CheckedBinaryOp { op, mode, left, right } => todo!(),
                    crate::xva::XvaOpcode::UnaryOp { op, left } => todo!(),
                    crate::xva::XvaOpcode::Read(xva_operand) => todo!(),
                    crate::xva::XvaOpcode::UMul { left, right } => todo!(),
                    crate::xva::XvaOpcode::SMul { left, right } => todo!(),
                };

                let nstat = XvaStatement::RawInstr(instr);

                if preamble.is_empty() {
                    *stmt = nstat
                } else {
                    preamble.push(nstat);

                    *stmt = XvaStatement::Elaborated(preamble)
                }
            },
            crate::xva::XvaStatement::Write(xva_operand, xva_type, xva_register) => todo!(),
            crate::xva::XvaStatement::Jump(symbol) => {
                let op = Operand::RelSymbol(RelocSym{sym: *symbol, kind: AddressKind::Default}, None);

                let instr = Instruction::new(SkyarchInstruction::JmpW { cond: SkyarchConditionCode::Always, link: SkyarchRegno::r0, dest: SkyarchRegno::r15 }, vec![op]);

                *stmt = XvaStatement::RawInstr(instr);
            },
            rstmt @ (crate::xva::XvaStatement::Tailcall { dest,  .. } |
            crate::xva::XvaStatement::Call { dest, .. }) => {
                let link = match rstmt {
                    XvaStatement::Tailcall { .. } => SkyarchRegno::r0,
                    _ => SkyarchRegno::r31,
                };
                let instr = match dest {
                    crate::xva::XvaOperand::Register(reg) => {
                        let reg = Self::areg(*reg);
                        Instruction::new_nullary(SkyarchInstruction::Jmpr { cond: SkyarchConditionCode::Always, link, dest: reg.regno() })
                    },
                    crate::xva::XvaOperand::Const(xva_const) => {
                        let opr = xva_const.to_direct_rel(AddressKind::Default, AddressKind::Default);
                        Instruction::new(SkyarchInstruction::JmpW { cond: SkyarchConditionCode::Always, link, dest: SkyarchRegno::r15 }, vec![opr])
                    },
                    crate::xva::XvaOperand::FrameAddr(_) => todo!(),
                };

                *stmt = XvaStatement::RawInstr(instr);
            },
            crate::xva::XvaStatement::Return => {
                *stmt = XvaStatement::RawInstr(Instruction::new_nullary(SkyarchInstruction::Jmpr { cond: SkyarchConditionCode::Always, link: SkyarchRegno::r0, dest: SkyarchRegno::r31 }))
            },
            crate::xva::XvaStatement::Trap(xva_trap) => {
                *stmt = XvaStatement::RawInstr(Instruction::new_nullary(SkyarchInstruction::Und00));
            },
            crate::xva::XvaStatement::Noop(_) => *stmt = XvaStatement::RawInstr(Instruction::new_nullary(SkyarchInstruction::Pause { k: 1 })),
            crate::xva::XvaStatement::RawInstr(_) |
            crate::xva::XvaStatement::OptGate(..) |
            crate::xva::XvaStatement::EndOptGate(_) |
            crate::xva::XvaStatement::Elaborated(..) |
            crate::xva::XvaStatement::Use(..) |
            crate::xva::XvaStatement::Fallthrough(..) => unimplemented!(),
        }
    }

    fn lower_epilogue(&self, frame: &crate::xva::XvaFrameProperties, _: Self::MachineMode) -> Vec<crate::xva::XvaStatement> {
        if !frame.has_prologue {
            return Vec::new();
        }

        let mut ret = Vec::new();

        let mut save_frame_size = -((frame.frame_size - 8 * (frame.is_leaf as usize)) as isize);

        if save_frame_size > 0 {
            let extended = save_frame_size < (i16::MIN as isize);
            ret.push(XvaStatement::RawInstr(Instruction::new_nullary(SkyarchInstruction::Addi { dest: SkyarchRegno::r30, signed: !extended, supress_flags: true, higher_half: false, imm: (save_frame_size as u16) })));
            if extended {
                ret.push(XvaStatement::RawInstr(Instruction::new_nullary(SkyarchInstruction::Addi { dest: SkyarchRegno::r30, signed: false, supress_flags: true, higher_half: true, imm: ((save_frame_size >> 16) as u16) })));
            }
        }

        if !frame.is_leaf {
            ret.push(XvaStatement::RawInstr(Instruction::new_nullary(SkyarchInstruction::Ld { dest: SkyarchRegno::r31, src: SkyarchRegno::r30, width: SkyarchByteSize::Word, mode: SkyarchLoadStoreMode::PostInc })));
        }

        ret

    }

    fn emit_prologue(&self, frame: &mut crate::xva::XvaFrameProperties, _: Self::MachineMode) -> Vec<crate::instr::Instruction> {
        let mut ret = Vec::new();
        frame.has_prologue = false;
        frame.frame_size = (frame.frame_size + (frame.frame_align - 1)) & !(frame.frame_align - 1);
        let save_frame_size = frame.frame_size;
        if !frame.is_leaf {
            frame.frame_size += 4;
            frame.has_prologue = true;
            ret.push(Instruction::new_nullary(SkyarchInstruction::St { dest: SkyarchRegno::r30, src: SkyarchRegno::r31, width: SkyarchByteSize::Word, mode: SkyarchLoadStoreMode::PreDec }));
        }

        if frame.frame_align > frame.call_align {
            todo!("Dynamic alignment")
        } else {
            frame.frame_align = frame.call_align;
        }

        if save_frame_size > 0 {
            frame.has_prologue = true;
            ret.push(Instruction::new_nullary(SkyarchInstruction::Addi { dest: SkyarchRegno::r30, signed: false, supress_flags: true, higher_half: false, imm: (save_frame_size as u16) }));
            if save_frame_size > (u16::MAX as usize) {
                ret.push(Instruction::new_nullary(SkyarchInstruction::Addi { dest: SkyarchRegno::r30, signed: false, supress_flags: true, higher_half: true, imm: ((save_frame_size >> 16) as u16) }));
            }
        }

        ret
    }
}