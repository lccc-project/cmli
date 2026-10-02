use std::num::NonZero;

use bitfield_struct::bitfield;

use crate::{archs::x86::{GprName, GprSize, X86, X86EncodingPrefix, X86Mode, X86Opcode, X86Register, XmmSize}, file::Encoder, instr::RelocSym, intern::Symbol, mach::MachineMode, reloc::{RelocSpan, RelocValue, RelocationKind}, traits::IdType as _};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
enum X86RegnoRInner {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7
}

impl core::fmt::Debug for X86RegnoRInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", *self as u8))
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct X86RegnoR(X86RegnoRInner);

impl X86RegnoR {
    pub const fn new(v: u8) -> Self {
        assert!(v < 8);

        unsafe { core::mem::transmute(v) }
    }

    pub const fn get(self) -> u8 {
        self.0 as u8
    }

    pub const fn from_bits(v: u8) -> Self {
        Self::new(v)
    }

    pub const fn into_bits(self) -> u8 {
        self.get()
    }

    pub const fn from_register(reg: X86Register) -> Self {
        Self::from_bits(reg.regno() & 7) // mask lower bits, encode upper bits in REX
    }

    pub const fn from_gpr(gpr: GprName) -> Self {
        Self::from_bits((gpr as u8) & 7)
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
enum X86RegnoVInner {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _18,
    _19,
    _20,
    _21,
    _22,
    _23,
    _24,
    _25,
    _26,
    _27,
    _28,
    _29,
    _30,
    _31,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct X86RegnoV(X86RegnoVInner);

impl X86RegnoV {
    pub const fn new(v: u8) -> Self {
        assert!(v < 32);

        unsafe { core::mem::transmute(v) }
    }

    pub const fn get(self) -> u8 {
        self.0 as u8
    }
    pub const fn from_register(reg: X86Register) -> Self {
        Self::new(reg.regno())
    }
}

impl core::fmt::Debug for X86RegnoVInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:o}", *self as u8))
    }
}



#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Disp {
    pub sym: Option<RelocSym>,
    pub disp: i32,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[bitfield_struct::bitenum]
#[repr(u8)]
pub enum Scale {
    #[fallback]
    Byte,
    Word,
    Double,
    Quad,
}

#[bitfield_struct::bitfield(u16, hash = true)]
pub struct Sib {
    #[bits(3)]
    pub base: X86RegnoR,
    #[bits(3)]
    pub index: X86RegnoR,
    #[bits(2)]
    pub scale: Scale,
    pub base_present: bool,
    pub index_present: bool,
    #[bits(6)]
    __: u8,
}

impl Sib {
    pub const fn reg_only(reg: X86Register) -> Sib {
        Sib::new().with_base(X86RegnoR::from_register(reg)).with_index(X86RegnoR(X86RegnoRInner::_4)).with_base_present(true)
    }

    pub const fn sib(scale: Scale, index: X86Register, base: X86Register) -> Sib{
        if matches!(index.regno(), GprName::SP_REGNO) {
            panic!("Cannot use rSP as an index register");
        }
        Sib::new().with_base(X86RegnoR::from_register(base)).with_index(X86RegnoR::from_register(index)).with_scale(scale).with_base_present(true).with_index_present(true)
    }

    pub const fn vsib(scale: Scale, index: X86Register, base: X86Register) -> Sib {
        Sib::new().with_base(X86RegnoR::from_register(base)).with_index(X86RegnoR::from_register(index)).with_scale(scale).with_base_present(true).with_index_present(true)
    }

    pub const fn index_only(scale: Scale, index: X86Register) -> Sib {
        if matches!(index.regno(), GprName::SP_REGNO) {
            panic!("Cannot use rSP as an index register");
        }
        Sib::new().with_base(X86RegnoR::new(5)).with_index(X86RegnoR::from_register(index)).with_scale(scale).with_index_present(true)
    }

    pub const fn vindex_only(scale: Scale, index: X86Register) -> Sib {
        Sib::new().with_base(X86RegnoR::new(5)).with_index(X86RegnoR::from_register(index)).with_scale(scale).with_index_present(true)
    }

    pub const fn sib_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl PartialEq for Sib {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Sib {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Rm {
    Reg(X86RegnoR),
    Abs(Disp),
    Rel(Disp),
    Mem(X86RegnoR, Option<Disp>),
    IndexLegacy(GprName, GprName, Option<Disp>),
    Sib(Sib, Option<Disp>),
}

#[bitfield_struct::bitfield(u8, hash = true, order = msb)]
#[allow(non_snake_case)]
pub struct Rex {
    /// Placeholder for if any feature of the generic REX prefix (e.g. new lower byte registers) is requested
    /// This is replaced by the computed REX prefix
    /// 
    /// This field has overloaded behaviour for encoding EVEX prefixes.
    /// If this field is set, then B, X, B4, X4, should be encoded in EVEX.B3, EVEX.X3, EVEX.B4, EVEX.X4
    /// If clear, then instead X/X4 are not encodable and B4 is encoded in EVEX.B4.
    /// 
    /// R and R4 are always encoded in EVEX.{R3, R4}.;
    /// 
    /// This behaviour does not alter the behaviour of VSIB, which always encodes B4 in EVEX.B4, and encodes X4 in EVEX.V4.
    /// 
    /// These behaviours overlap, as only uses of GPRs (in the R/M field) can set this in an EVEX instruction
    pub use_misc: bool,
    #[bits(1)]
    pub R4: u8,
    #[bits(1)]
    pub X4: u8,
    #[bits(1)]
    pub B4: u8,
    pub W: bool,
    #[bits(1)]
    pub R: u8,
    #[bits(1)]
    pub X: u8,
    #[bits(1)]
    pub B: u8,
}

impl Rex {
    pub fn set_base(&mut self, reg: X86Register) {
        let regno = reg.regno();

        self.set_B((regno >> 3) & 1);
        self.set_B4((regno >> 4) & 1);
    }

    pub fn set_index(&mut self, reg: X86Register) {
        let regno = reg.regno();

        self.set_X((regno >> 3) & 1);
        self.set_X4((regno >> 4) & 1);
    }

    pub fn set_reg(&mut self, reg: X86Register) {
        let regno = reg.regno();

        self.set_R((regno >> 3) & 1);
        self.set_R4((regno >> 4) & 1);
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum EvexKind {
    Legacy,
    Vidx,
    Apx,
}

impl Rex {
    pub const fn is_rex2(&self) -> bool {
        self.0 & 0x70 != 0
    }

    pub const fn need_rex(&self) -> bool {
        self.0 != 0
    }

    pub const fn into_rex(self, w: bool) -> u8 {
        0x40 | ((w as u8) << 3) | self.0 & 0x0F
    }

    pub const fn into_rex2(self, w: bool, map: X86Map) -> u8  {
        (self.0 & 0x7F) | ((w as u8) << 3) | match map {
            X86Map::Base => 0x00,
            X86Map::Map0F => 0x80,
            X86Map::Map0F38 => panic!("Cannot encode 0x0F 0x38 map instruction with REX2"),
            X86Map::Map0F3A => panic!("Cannot encode 0x0F 0x3A map instruction with REX2"),
            X86Map::VexMap0 => panic!("Cannot encode Mandatory VEX instruction instruction with REX2"),
            X86Map::VexMap7 => panic!("Cannot encode Mandatory VEX instruction instruction with REX2"),
            X86Map::EvexMap5 => panic!("Cannot encode Mandatory EVEX instruction instruction with REX2"),
            X86Map::EvexMap6 => panic!("Cannot encode Mandatory EVEX instruction instruction with REX2"),
        }
    }

    pub const fn into_vex(self, w: bool, map: X86Map) -> Vex {
        let map = map.vex_mapno();
        
        if self.is_rex2() {
            panic!("Cannot encode >16 registers with VEX");
        }

        let rex = (!(self.0 & 7)) << 5;

        let w = ((self.W() | w) as u8) << 7;


        Vex(u16::from_be_bytes([rex | map, w]))
    }

    pub const fn into_evex(self, w: bool, map: X86Map, sib: Option<SibKind>) -> Evex {
        let map = map.vex_mapno();

        assert!(map < 8);
        
        let mut rex_base = (!(self.0 & 7)) << 5 | ((!self.R4()) & 1) << 4;

        let mut width = ((self.W() | w) as u8) << 7;

        let mut byte3 = 0;

        match sib {
            Some(SibKind::Sib) => {
                rex_base |= self.B4() << 3;
                width |= ((!self.X4()) & 1) << 2;
            },
            Some(SibKind::Vsib) => {
                assert!(self.B4() != 0);

                byte3 |= ((!self.X4()) & 1) << 3;
            }
            None => {
                assert!(self.X() == 0 && self.X4() == 0);
                rex_base |= ((!self.B4()) & 1) << 6;
            },
        }


        Evex(u32::from_be_bytes([0x62, rex_base | map, width, byte3]))
    }
}

impl PartialEq for Rex {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Rex {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ModRM {
    pub rex: Option<Rex>,
    pub reg: X86RegnoR,
    pub rm: Rm,
}

impl ModRM {
    /// Encodes the ModR/M byte according to mode
    /// This encodes a 32/64-bit ModR/M regardless of mode. `mode` controls whether or not ModR/M `0o005` is interpreted as `[disp32]` or `[rip + disp32]`.
    /// 
    /// # Panics
    /// Panics if the [`Rm`] field is invalid for the mode and address type (e.g. encoding [`Rm::Rel`] in legacy mode, or encoding an [`Rm::IndexLegacy`])
    pub fn encode_modrm(&self, mode: X86Mode) -> u8 {
        (self.reg.get() << 3) | match (self.rm, mode) {
            (Rm::Reg(r), _) => 0o300 | r.get(),
            (Rm::Abs(_), X86Mode::Long) => 0o004, // Encode Absolute as SIB in Long Mode
            (Rm::Rel(_), X86Mode::Long) => 0o005,
            (Rm::Abs(_), _) => 0o005,
            (Rm::Rel(_), _) => panic!("Cannot encode relative modR/M in legacy mode"),
            (Rm::IndexLegacy(_, _, _), _) => panic!("Cannot encode legacy index in 32-bit mode"),
            (Rm::Mem(r, disp), _) => {
                let mode = match disp {
                    None if r.get() == 5 => 0o100,
                    None => 0o000,
                    Some(Disp { sym: None, disp: -128..128 }) => 0o100,
                    _ => 0o200,
                };

                mode | r.get()
            },
            (Rm::Sib(sib, disp), _) => {
                match disp {
                    _ if !sib.base_present() => 0o004, // base absent needs mode 0
                    None if sib.base().get() == 5 => 0o104,
                    None => 0o004,
                    Some(Disp { sym: None, disp: -128..128 }) => 0o104,
                    _ => 0o204,
                }
            },
            
        }
    }

    /// Encodes the 16-bit ModR/M. This assumes legacy mode (as encoding a 16-bit ModR/M in Long mode is impossible)
    /// 
    /// # Panics
    /// Panics if the [`Rm`] field is invalid for the address type.
    pub fn encode_modrm16(&self) -> u8 {
        (self.reg.get() << 3) | match self.rm {
            Rm::Reg(r) => todo!(),
            Rm::Abs(disp) => 0o005,
            Rm::Rel(disp) => panic!("Cannot encode relative modR/M in legacy mode"),
            Rm::Mem(r, disp) => {

                let maddr = match GprName::from_regno(r.get()){
                    GprName::bx => 7,
                    GprName::bp => 6,
                    GprName::si => 4,
                    GprName::di => 5,
                    r => panic!("Cannot encode memory reference to {}", r.as_reg(GprSize::Word))
                };

                let mode = match disp {
                    None if r.get() == 5 => 0o100,
                    None => 0o000,
                    Some(Disp { sym: None, disp: -128..128 }) => 0o100,
                    _ => 0o200,
                };

                mode | maddr
            },
            Rm::IndexLegacy(base, index, disp) => {
                let maddr = match (base, index) {
                    (GprName::bx, GprName::si) => 0,
                    (GprName::bx, GprName::di) => 1,
                    (GprName::bp, GprName::si) => 2,
                    (GprName::bp, GprName::di) => 3,
                    (base, index) => panic!("Cannot encode memory reference to {}+{}", base.as_reg(GprSize::Word), index.as_reg(GprSize::Word)),
                };

                let mode = match disp {
                    None => 0o000,
                    Some(Disp { sym: None, disp: -128..128 }) => 0o100,
                    _ => 0o200,
                };

                mode | maddr
            },
            Rm::Sib(sib, disp) => panic!("Cannot encode SIB for a 16-bit address"),
        }
    }

    /// Gets the actual displacement of the [`ModRM`]. If [`Some`], the first value is the value to be encoded, and the second is the width of the displacement in bits.
    /// `dwidth` is the "normal" displacement width: 32 for 32-bit and 64-bit addresses, 16 for 16-bit addresses.
    pub fn get_disp(&self, dwidth: usize) -> Option<(i32, usize)> {
        match self.rm {
            Rm::Reg(_) => None,
            Rm::Abs(disp) |
            Rm::Rel(disp) => Some((disp.disp, dwidth)),
            Rm::Mem(r, disp) => {
                match disp {
                    None if r.get() == 5 => Some((0, 8)),
                    None => None,
                    Some(Disp{sym: None, disp: disp @ (-128..128)}) => Some((disp, 8)),
                    Some(Disp{disp, ..}) => Some((disp, dwidth)),
                }
            },
            Rm::IndexLegacy(_, _, disp) => {
                match disp {
                    Some(Disp{sym: None, disp: disp @ (-128..128)}) => Some((disp, 8)),
                    Some(Disp{disp, ..}) => Some((disp, dwidth)), 
                    None => todo!(),
                }
            },
            Rm::Sib(sib, disp) => {
                match disp {
                    _ if !sib.base_present() => Some((0, dwidth)),
                    None if sib.base().get() == 5 => Some((0, 8)),
                    None => None,
                    Some(Disp{sym: None, disp: disp @ (-128..128)}) => Some((disp, 8)),
                    Some(Disp{disp, ..}) => Some((disp, dwidth)),
                }
            },
        }
    }

    pub fn get_sym(&self) -> Option<RelocSym> {
        match self.rm {
            Rm::Reg(_) => None,
            Rm::Abs(disp) |
            Rm::Rel(disp) => disp.sym,
            Rm::Mem(_, disp) |
            Rm::IndexLegacy(_, _, disp) |
            Rm::Sib(_, disp) => disp.and_then(|r| r.sym),
        }
    }

    /// Encodes the ModR/M byte according to mode
    /// This encodes a 32/64-bit ModR/M regardless of mode, as SIB indexing is not available for 16-bit addresses (use [`Rm::IndexLegacy`] instead). 
    /// `mode` controls whether or not ModR/M `0o005` is interpreted as `[disp32]` or `[rip + disp32]` (notably, it controls whether or not an SIB byte is encoded for `[disp32]`)
    /// 
    /// # Panics
    /// Panics if the [`Rm`] field is invalid for the mode and address type (e.g. encoding [`Rm::Rel`] in legacy mode, or encoding an [`Rm::IndexLegacy`])
    pub fn get_sib(&self, mode: X86Mode) -> Option<u8> {
        match (self.rm, mode) {
            (Rm::Reg(_), _) => None,
            (Rm::Rel(_), X86Mode::Long) => None,
            (Rm::Abs(_), X86Mode::Long) => Some(0o045),
            (Rm::Rel(_), _) => panic!("Cannot encode Relative Address in Legacy Mode"),
            (Rm::Abs(_), _) => None,
            (Rm::Mem(r, _), _) => {
                if r.get() == 4 {
                    Some(0o44)
                } else {
                    None
                }
            }
            (Rm::Sib(sib, _), _) => Some(sib.sib_byte()),
            (Rm::IndexLegacy(_, _, _), _) => panic!("Cannot encode legacy indexing in a 32-bit address")
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum X86OpEncoding {
    None,
    OpReg(Rex, X86RegnoR),
    ModRM(ModRM),
    Vex(XmmSize, Option<X86RegnoV>, ModRM),
    VexIs4(XmmSize, X86RegnoV, X86RegnoV, ModRM),
    Evex(EvexExtra, ModRM),
    AbsPtr(u16),
    RelImm,

}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SibKind {
    Sib,
    Vsib,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum EvexExtra {
    /// Basic AVX-512 encoding, or APX Exnteded AVX-512 (only when X4 or B4 are set). Instructions that use {x,y,z}mm16-32 encode the upper bit of base in X3 if there is no SIB byte
    Avx512 {
        width: XmmSize,
        k: X86RegnoR,
        z: bool,
        b: bool,
        vec: Option<X86RegnoV>,
    },
    /// Same as [`EvexExtra::Avx512`] but does not encode `vec`, and encodes X4 in V4. B4 is not encodeable in this mode.
    EvexVidx {
        width: XmmSize,
        k: X86RegnoR,
        z: bool,
        b: bool,
    },
    /// APX Extended VEX prefix.
    ExtVex {
        width: XmmSize,
        nf: bool,
        vec: Option<X86RegnoV>,
    },
    /// APX Extended Legacy Prefix.
    ExtLegacy {
        nf: bool,
        ndd: Option<X86RegnoV>,
    },
    /// APX CCMP/CTEST prefix encoding
    Ccmp {
        cc: u8,
        flags: u8,
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum SsePrefix {
    None = 0,
    Pd = 0x66,
    Ss = 0xF3,
    Sd = 0xF2,
}

impl SsePrefix {

    pub const fn into_bits(self) -> u8 {
        self.into_vex_p()
    }

    pub const fn from_bits(v: u8) -> Self {
        match v {
            0 => SsePrefix::None,
            1 => SsePrefix::Pd,
            2 => SsePrefix::Ss,
            3 => SsePrefix::Sd,
            _ => panic!("Invalidd SSE Prefix")
        }
    }

    pub const fn into_vex_p(self) -> u8 {
        match self {
            SsePrefix::None => 0,
            SsePrefix::Pd => 1,
            SsePrefix::Ss => 2,
            SsePrefix::Sd => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u16)]
pub enum X86Map {
    Base = 0,
    Map0F = 0x0F,
    Map0F38 = 0x0F38,
    Map0F3A = 0x0F3A,

    // Illegal prefixes, that require VEX or EVEX prefixes to encode
    VexMap0 = 0xFF00,
    VexMap7 = 0xFF07,

    EvexMap5 = 0xFF05,
    EvexMap6 = 0xFF06,
}

impl X86Map {
    pub const fn vex_mapno(&self) -> u8 {
        match self {
            X86Map::Base => 4,
            X86Map::Map0F => 1,
            X86Map::Map0F38 => 2,
            X86Map::Map0F3A => 3,
            X86Map::VexMap0 => 0,
            X86Map::VexMap7 => 7,
            X86Map::EvexMap5 => 5,
            X86Map::EvexMap6 => 6,
        }
    }

    pub const fn into_bits(self) -> u8 {
        self.vex_mapno()
    }

    pub const fn from_bits(v: u8) -> Self {
        match v {
            4 => X86Map::Base,
            1 => X86Map::Map0F,
            2 => X86Map::Map0F38,
            3 => X86Map::Map0F3A,
            0 => X86Map::VexMap0,
            7 => X86Map::VexMap7,
            5 => X86Map::EvexMap5,
            6 => X86Map::EvexMap6,
            _ => panic!("Unknown map")
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum LegacyPrefix {
    Lock = 0xF0,
    Repnz = 0xF2,
    Repz = 0xF3,
    FWait = 0x9B,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum SegmentOverride {
    ES = 0x26,
    CS = 0x2E,
    SS = 0x36,
    DS = 0x3E,
    FS = 0x64,
    GS = 0x65,
}

impl SegmentOverride {
    pub const BRANCH_NOT_TAKEN: Self = Self::CS;
    pub const BRANCH_TAKEN: Self = Self::DS;

    pub fn from_sreg(r: X86Register) -> Self {
        match r {
            X86Register::Segment(r) => {
                [SegmentOverride::ES, SegmentOverride::CS, SegmentOverride::DS, SegmentOverride::SS, SegmentOverride::FS, SegmentOverride::GS][r as usize]
            }
            _ => panic!("{r} is not an segment register")
        }
    }
}

trait FetchIncrement {
    fn fetch_inc(&mut self) -> Self;
}

impl FetchIncrement for usize {
    fn fetch_inc(&mut self) -> Self {
        let val = *self;
        *self += 1;
        val
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct X86Instruction {
    pub mode: X86Mode,
    pub legacy_prefix: Option<LegacyPrefix>,
    pub sprefix: Option<SegmentOverride>,
    pub op_size: Option<GprSize>,
    pub default_op_size: Option<GprSize>,
    pub addr_size: GprSize,
    pub map: X86Map,
    pub sse_prefix: Option<SsePrefix>,
    pub opcode: u8,
    pub ops: X86OpEncoding,
    pub imm: Option<(i64, u32)>,
    pub immsym: Option<RelocSym>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct InvBits<const M: u8>(pub u8);

impl<const M: u8> InvBits<M> {
    pub const fn from_bits(v: u8) -> Self {
        Self((!v) & M)
    }

    pub const fn into_bits(self) -> u8 {
        (!self.0) & M
    }
}

#[bitfield_struct::bitfield(u16, hash = true, order = msb)]
#[allow(non_snake_case)]
pub struct Vex {
    #[bits(3)]
    pub rex: InvBits<7>,
    #[bits(5)]
    pub map: X86Map,
    #[bits(1)]
    pub W: u8,
    #[bits(4)]
    pub v: InvBits<4>,
    #[bits(1)]
    pub L: bool,
    #[bits(2)]
    pub pfx: SsePrefix,
}

#[bitfield_struct::bitfield(u32, hash = true, order = msb)]
#[allow(non_snake_case)]
pub struct Evex {
    #[bits(8)]
    __: u8,
    #[bits(3)]
    pub rex: InvBits<7>,
    #[bits(1)]
    pub R4: InvBits<1>,
    #[bits(1)]
    pub B4: u8,
    #[bits(3)]
    pub map: X86Map,
     #[bits(1)]
    pub W: bool,
    #[bits(4)]
    pub v: InvBits<0xF>,
    #[bits(1)]
    pub X4: InvBits<1>,
    #[bits(2)]
    pub p: SsePrefix,
    pub z: bool,
    #[bits(2)]
    pub L: u8,
    pub b: bool,
    #[bits(1)]
    pub v4: InvBits<1>,
    #[bits(3)]
    pub a: X86RegnoR,
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum EncodingPrefix {
    None,
    Rex(Rex),
    Vex(Vex),
    Evex(Evex),
}

impl X86Instruction {
    /// Encodes an x86 instruction into v and returns the instruction
    /// 
    pub fn encode<'a>(&self, enc: &'a mut [u8], mut reloc: impl FnMut(RelocValue, isize)) -> &'a mut [u8] {
        let mut pos = 0;
        if let Some(p) = self.legacy_prefix {
            enc[pos.fetch_inc()] = p as u8
        }


        if let Some(p) = self.sprefix {
            enc[pos.fetch_inc()] = p as u8;
        }

        let mut rex_w = false;

        match (self.op_size, self.default_op_size, self.mode) {
            (Some(GprSize::Byte) | None, _, _) => {}
            (Some(GprSize::Word), _, X86Mode::Real | X86Mode::Protected16) => {}
            (Some(GprSize::Double), Some(GprSize::Quad), X86Mode::Long) => panic!("Cannot encode 32-bit operand for instruction in this mode"),
            (Some(GprSize::Double), _, X86Mode::Long | X86Mode::Protected) => {}
            (Some(GprSize::Double), _, X86Mode::Real | X86Mode::Protected16) | (Some(GprSize::Word), _, _) => {
                enc[pos.fetch_inc()] = 0x66;
            }
            (Some(GprSize::Quad), Some(GprSize::Quad), X86Mode::Long) => {}
            (Some(GprSize::Quad), _, X86Mode::Long) => {
                rex_w = true;
            }
            (Some(GprSize::Quad), _, _) => panic!("Cannot encode 64-bit operand in this mode")
        }

        match (self.addr_size, self.mode) {
            (GprSize::Word, X86Mode::Real | X86Mode::Protected16) => {}
            (GprSize::Double, X86Mode::Real | X86Mode::Protected16 | X86Mode::Long) => {
                enc[pos.fetch_inc()] = 0x67;
            }
            (GprSize::Word, X86Mode::Protected) => {
                enc[pos.fetch_inc()] = 0x67;
            }
            (GprSize::Quad, X86Mode::Long) => {
                
            }
            _ => {
                panic!("Cannot encode {}-bit addresses in mode", self.addr_size.bits());
            }
        }

        let mut opc = self.opcode;

        let (mut imm, mut immsize) = self.imm.unwrap_or((0, 0));

        let mut imm_sym = self.immsym;

        let mut ptr_seg = None::<u16>;

        let (pfx, modrm) = match self.ops {
            X86OpEncoding::None => (EncodingPrefix::None, None),
            X86OpEncoding::OpReg(rex, r) => {
                opc += r.get();

                (EncodingPrefix::Rex(rex), None)
            },
            X86OpEncoding::ModRM(modrm) => {
                (EncodingPrefix::Rex(modrm.rex.unwrap_or(const { Rex::new() })), Some(modrm))
            },
            X86OpEncoding::Vex(sz, v, modrm) => {
                let rex = modrm.rex.unwrap_or(Rex::new());

                let mut vex = rex.into_vex(rex_w, self.map);

                match sz {
                    XmmSize::Xmm => vex.set_L(false),
                    XmmSize::Ymm => vex.set_L(true),
                    XmmSize::Zmm => panic!("Cannot encode zmm regs with VEX"),
                }

                match v {
                    Some(v) => {
                        let v = v.get();

                        if v > 15 {
                            panic!("Cannot encode {sz}{v} with VEX");
                        }

                        vex.set_v(InvBits(v));
                    }
                    None => {}
                }

                (EncodingPrefix::Vex(vex), Some(modrm))
            },
            X86OpEncoding::VexIs4(sz, v, is4, modrm) => {
                let rex = modrm.rex.unwrap_or(Rex::new());

                let mut vex = rex.into_vex(rex_w, self.map);

                match sz {
                    XmmSize::Xmm => vex.set_L(false),
                    XmmSize::Ymm => vex.set_L(true),
                    XmmSize::Zmm => panic!("Cannot encode zmm regs with VEX"),
                }

               let v = v.get();

                if v > 15 {
                    panic!("Cannot encode {sz}{v} with VEX");
                }

                vex.set_v(InvBits(v));

                let is4 = is4.get();

                if is4 > 15 {
                    panic!("Cannot encode {sz}{v} with VEX");
                }

                imm |= (is4 << 4) as i64;
                assert!(immsize <= 4);
                immsize = 8; // hardcode immsize to 8

                (EncodingPrefix::Vex(vex), Some(modrm))
            },
            X86OpEncoding::Evex(evex, modrm) => {
                let has_mem = match modrm.rm {
                    Rm::Reg(_) => false,
                    _ => true,
                };

                let rex = modrm.rex.unwrap_or(const { Rex::new() });
                let evex = match evex {
                    EvexExtra::Avx512 { width, k, z, b, vec } => {
                        let mut evex = rex.into_evex(rex_w, self.map, if has_mem { Some(SibKind::Sib) } else { None});

                        evex.set_a(k);
                        evex.set_z(z);
                        
                        match width {
                            XmmSize::Xmm => evex.set_L(0),
                            XmmSize::Ymm => evex.set_L(1),
                            XmmSize::Zmm => evex.set_L(2),
                        }

                        if let Some(vec) = vec {
                            let v = vec.get();
                            let v4 = v >> 4;
                            let v = v & 0xF;

                            evex.set_v(InvBits(v));
                            evex.set_v4(InvBits(v4));
                        } else {
                            evex.set_v4(InvBits(0));
                        }

                        evex
                    },
                    EvexExtra::EvexVidx { width, k, z, b } => todo!(),
                    EvexExtra::ExtVex { width, nf, vec } => todo!(),
                    EvexExtra::ExtLegacy { nf, ndd } => todo!(),
                    EvexExtra::Ccmp { cc, flags } => todo!(),
                };


                (EncodingPrefix::Evex(evex), Some(modrm))
            },
            X86OpEncoding::RelImm =>(EncodingPrefix::None, None),
            X86OpEncoding::AbsPtr(seg) => {
                ptr_seg = Some(seg);
                (EncodingPrefix::None, None)
            }
        };

        match pfx {
            EncodingPrefix::None => {
                if rex_w {
                    // encode REX.W
                    enc[pos.fetch_inc()] = 0x48; // REX.W hardcoded
                }
                if let Some(pfx) = self.sse_prefix {
                    enc[pos.fetch_inc()] = pfx as u8;
                }

                match self.map {
                    X86Map::Base => {},
                    X86Map::Map0F => enc[pos.fetch_inc()] = 0x0F,
                    X86Map::Map0F38 => {
                        enc[pos.fetch_inc()] = 0x0F;
                        enc[pos.fetch_inc()] = 0x38;
                    },
                    X86Map::Map0F3A => {
                        enc[pos.fetch_inc()] = 0x0F;
                        enc[pos.fetch_inc()] = 0x3A;
                    },
                    _ => panic!("Cannot encode Map {:#.4X} without VEX or EVEX", self.map as u16)
                }
            },
            EncodingPrefix::Rex(rex) => {
                if let Some(pfx) = self.sse_prefix {
                    enc[pos.fetch_inc()] = pfx as u8;
                }

                if rex.is_rex2() {
                    let rex2 = rex.into_rex2(rex_w, self.map);

                    enc[pos.fetch_inc()] = 0xD5;
                    enc[pos.fetch_inc()] = rex2;
                } else {
                    if rex.need_rex() || rex_w {
                        enc[pos.fetch_inc()] = rex.into_rex(rex_w);
                    }
                    match self.map {
                        X86Map::Base => {},
                        X86Map::Map0F => enc[pos.fetch_inc()] = 0x0F,
                        X86Map::Map0F38 => {
                            enc[pos.fetch_inc()] = 0x0F;
                            enc[pos.fetch_inc()] = 0x38;
                        },
                        X86Map::Map0F3A => {
                            enc[pos.fetch_inc()] = 0x0F;
                            enc[pos.fetch_inc()] = 0x3A;
                        },
                        _ => panic!("Cannot encode Map {:#.4X} without VEX or EVEX", self.map as u16)
                    }
                }
            },
            EncodingPrefix::Vex(vex) => {
                let bytes = vex.0.to_be_bytes();

                if bytes[0] & 0x7F == 0x61 && bytes[1] & 0x80 == 0 {
                    enc[pos.fetch_inc()] = 0xC5;
                    enc[pos.fetch_inc()] = (bytes[0] & 0x80) | (bytes[1] & 0x7F);
                } else {
                    enc[pos.fetch_inc()] = 0xC4;
                    enc[pos..(pos+2)].copy_from_slice(&bytes);

                    pos += 2;
                }
            },
            EncodingPrefix::Evex(evex) => {
                let bytes = evex.0.to_be_bytes();

                enc[pos..(pos+4)].copy_from_slice(&bytes);

                pos += 4;
            },
        }

        enc[pos.fetch_inc()] = opc;

        match modrm {
            Some(modrm) => {
                let dwidth = match self.addr_size {
                    GprSize::Double | GprSize::Quad => {
                        let byte = modrm.encode_modrm(self.mode);

                        enc[pos.fetch_inc()] = byte;

                        if let Some(sib) = modrm.get_sib(self.mode) {
                            enc[pos.fetch_inc()] = sib;
                        }

                        32
                    }
                    GprSize::Word => {
                        let byte = modrm.encode_modrm16();

                        16
                    }
                    _ => unreachable!("Illegal Address Size")
                };

                if let Some((disp, width)) = modrm.get_disp(dwidth) {
                    assert!(immsize <= 32, "Displacement not allowed for 64-bit immediate");

                    let rel = matches!(modrm.rm, Rm::Rel(_));

                    if let Some(sym) = modrm.get_sym() {
                        reloc(RelocValue { 
                            sym: Some(sym.sym), 
                            addend: disp as i64, 
                            kind: sym.kind.into_reloc(RelocSpan{byte_width: (width/8) as u8, bit_width: width as u8, overflow_kind: crate::reloc::OverflowKind::Signed, ..RelocSpan::new()}, rel)
                        }, pos as isize);
                    }

                    let bytes = width / 8;

                    let bits = disp.to_le_bytes();

                    enc[pos..(pos+bytes)].copy_from_slice(&bits[..bytes]);

                    pos += bytes;

                }
            },
            None => {},
        }


        let rel = matches!(self.ops, X86OpEncoding::RelImm);

        if let Some(ptr) = ptr_seg {
            let bytes = ptr.to_le_bytes();
            enc[pos.fetch_inc()] = bytes[0];
            enc[pos.fetch_inc()] = bytes[1];
        }

        let bytes = ((immsize + 7) / 8) as usize;

        if let Some(sym) = imm_sym {
            reloc(RelocValue { 
                sym: Some(sym.sym), 
                addend: imm as i64, 
                kind: sym.kind.into_reloc(RelocSpan{byte_width: bytes as u8, bit_width: immsize as u8, overflow_kind: crate::reloc::OverflowKind::Signed, ..RelocSpan::new()}, rel)
            }, pos as isize);
        }

        let bits = imm.to_le_bytes();

        enc[pos..(pos+bytes)].copy_from_slice(&bits[..bytes]);

        pos += bytes;


        &mut enc[..pos]
    }
}


impl Encoder for X86 {
    fn encode_instr(&self, writer: &mut crate::file::Section, instr: &crate::instr::Instruction, mode: MachineMode) -> std::io::Result<usize> {
        let mode = mode.downcast::<X86Mode>().expect("Wrong Type");
        let instr = X86Opcode::encode(&instr, mode);

        let mut err = Ok(());

        let mut buf = [0u8; 16];

        let instr = instr.encode(&mut buf, |val, off| {
            let r = core::mem::replace(&mut err, Ok(()));
            err = r.and_then(|_| writer.write_relocation(off, val));
        });

        err.and_then(|_| writer.write_bytes(instr)).map(|_| instr.len())
    }
}