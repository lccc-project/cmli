//! # x86 Support
//! 
//! x86 is supported in 16-bit, 32-bit, and 64-bit mode.



use crate::{
    compiler::CompilerSpec, instr::{AddressKind, Instruction, Operand, RelocSym}, mach::{MachineMode, MachineSpec, Opcode, Register, RegisterSpec}, traits::{AsId, AsRawId, IdType, Name}, xva::{BinaryOp, XvaCategory, XvaExpr, XvaOpcode, XvaRegister, XvaStatement}
};

use crate::instr::RegisterKind;

/// The machine mode for X86.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId, Name)]
pub enum X86Mode {
    /// 16-bit real mode
    Real,
    /// 16-bit protected mode. This differs from [`X86Mode::Real`] in that certain registers are only available in this mode
    Protected16,
    /// 32-bit protected mode
    Protected,
    /// 64-bit long mode
    Long,
}

impl X86Mode {
    const ALL_MODES: [X86Mode; 4] = [
        X86Mode::Real,
        X86Mode::Protected16,
        X86Mode::Protected,
        X86Mode::Long,
    ];

    /// Largest supported [`GprSize`] in the current mode
    pub fn largest_gpr(&self) -> GprSize {
        match self {
            X86Mode::Real => GprSize::Word,
            X86Mode::Protected16 => GprSize::Word,
            X86Mode::Protected => GprSize::Double,
            X86Mode::Long => GprSize::Quad,
        }
    }

    /// Convenience function for determining if the current mode supports relative addresses in ModR/M bytes. This is only true on [`X86Mode::Long`]
    pub fn supports_rel_addr(&self) -> bool {
        matches!(self, X86Mode::Long)
    }
}

impl const AsId<MachineMode> for X86Mode {}

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
        $(#[$meta:meta])*
        $vis:vis enum $name:ident ($class_enum:ident) {
            $($(#[$var_meta:meta])* $class:ident $(#[norex] $(@ $_norex_tt:tt)?)? [ $($names:ident),* $(, #$prefix:ident _ $($suffix:ident)? $begin:literal..$end:literal)?]  $(($size:literal))? @ $category:ident $(($($cat_tt:tt)*))? $(overlaps [$($overlap_var:ident),*])? = $kind:expr),* $(,)?
        }
    } => {
        #[derive(Copy, Clone, Hash, PartialEq, Eq, AsRawId)]
        #[repr(u8)]
        #[non_exhaustive]
        $(#[$meta])*
        $vis enum $name {
            $($(#[$var_meta])* $class (u8)),*
        }

        const _: () = {
            #[repr(C)]
            struct __Concat<$($class),*>($($class),*);

            const fn __count_by_class(__reg: $name) -> usize {
                match __reg {
                    $($name :: $class (_) => ($($(@ $_norex_tt)? 8,)? $(${ignore($prefix)} 32,)? ${count($names)},).0),*
                }
            }

            const __TOTAL_COUNT: usize = $(__count_by_class($name :: $class (0)) +)* 0;

            const fn __make_array<const N: usize, const __CLASS: $class_enum>() -> [$name; N] {
                let mut ret: [$name; N] = unsafe { core::mem::zeroed() };

                let mut i = 0;

                while i < N {
                    ret[i] = match __CLASS { $($class_enum::$class => $name::$class (i as u8)),* };
                    i += 1;
                }

                ret
            }

            const fn __concat<const N: usize, $($class),*>(__helper: __Concat<$($class),*>) -> [$name; N] {
                unsafe { $crate::mem::transmute(__helper) }
            }

            impl $name {
                const ALL_REGISTERS: [$name; __TOTAL_COUNT] = __concat(__Concat(
                    $(__make_array::<{ __count_by_class($name :: $class (0)) }, { $class_enum :: $class}>()),*)
                );
            }
        };

        $(#[$meta])*
        #[derive(Copy, Clone, Hash, PartialEq, Eq, core::marker::ConstParamTy)]
        #[non_exhaustive]
        $vis enum $class_enum {
            $($(#[$var_meta])* $class),*
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

            /// Converts the register from the specified name
            #[inline]
            pub fn from_name(name: &str) -> Self {
                Self::from_name_impl(name)
            }

            /// Obtains the class corresponding to the register
            pub const fn class(&self) -> $class_enum {
                match self {
                    $(Self:: $class(_) => $class_enum :: $class),*
                }
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
            const fn kind(&self) -> RegisterKind {
                match self {
                    $(Self::$class(_) => $kind),*
                }
            }

            fn size(&self, m: X86Mode) -> u32 {
                match self {
                    $(Self::$class(_) => ($($size,)? m.largest_gpr().bits() / 8,).0),*
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

            /// Extracts the register number for the register
            pub const fn regno(&self) -> u8 {
                match self {
                    $(Self::$class(__n) => *__n $(& 7 $(@ $_norex_tt)?)?),*
                }
            }

            /// Extracts the category of the register class
            pub const fn category(&self) -> crate::xva::XvaCategory {
                match self {
                    $(Self::$class(_) => crate::xva::XvaCategory:: $category $(($($cat_tt)*))?,)*
                }
            }
        }
    }
}

define_x86_registers! {
    /// X86 Registers
    pub enum X86Register (X86RegisterClass) {
        /// Byte Type registers
        /// This only supports the first 4 registers of this kind and unifies [`X86RegisterClass::ByteLegacy`] and [`X86RegisterClass::ByteRex`] for these register types
        Byte [al, cl, dl, bl] (1) @ Int = RegisterKind::GeneralPurpose,
        /// Legacy Byte Registers. These access the low bytes (0..4) or high bytes (4..8) of ax, cx, dx, and bx.
        /// They cannot access sp, bp, si, or di, or any extended (Long Mode or apx) GPR
        /// Due to encoding characteristics, the high byte registers cannot be used in any instruction that uses a REX, REX2, or Extended EVEX prefix of any kind
        ByteLegacy #[norex][al, cl, dl, bl, ah, ch, dh, bh] (1) @ Int = RegisterKind::GeneralPurpose,
        /// REX Promoted Byte Registers. These access the low bytes of the corresponding register.
        /// These are only accessible in [`X86Mode::Long`]
        ByteRex[al, cl, dl, bl, spl, bpl, sil, dil, #r _ b 8..32] (1) @ Int overlaps [Word, Double, Quad] = RegisterKind::GeneralPurpose,
        /// Word registers. These access the lower 2 bytes of the full gpr
        Word[ax, cx, dx, bx, sp, bp, si, di, #r _ w 8..32] (2) @ Int overlaps [ByteRex, Double, Quad] = RegisterKind::GeneralPurpose,
        /// Doubleword registers. These access the lower 4 bytes of the full gpr. These are not accessible on [`X86Mode::Real`] (but are in [`X86Mode::Protected16`])
        Double[eax, ecx, edx, ebx, esp, ebp, esi, edi, #r _ d 8..32] (4) @ Int overlaps [ByteRex, Word, Quad] = RegisterKind::GeneralPurpose,
        /// Quadword registers. These access the full GPR. These are only accesible in [`X86Mode::Long`]
        Quad [rax, rcx, rdx, rbx, rsp, rbp, rsi, rdi, #r _ 8..32] (8) @ Int overlaps [ByteRex, Word, Double] = RegisterKind::GeneralPurpose,
        /// Segment Registers. 
        Segment #[norex] [es, cs, ss, ds, fs, gs] (2) @ Custom(RegisterKind::AddressSegment) = RegisterKind::AddressSegment,
        /// Floating point stack registers. Note that these correspond to the synthetic registers exposed by fcw.TOP. There is no correspondance for the real fp registers as fp registers
        St [, #st _ 0..8] (10) @ Float = RegisterKind::ScalarFp,
        /// MMX Technology Registers. 
        Mmx #[norex] [, #mm _ 0..8] (8) @ VectorInt = RegisterKind::VectorInt,
        /// SSE 16-byte xmm registers
        Xmm [, #xmm _ 0..32] (16) @ VectorAny overlaps [Ymm, Zmm] = RegisterKind::VectorAny,
        /// AVX 32-byte ymm registers
        Ymm [, #ymm _ 0..32] (32) @ VectorAny overlaps [Xmm, Zmm] = RegisterKind::VectorAny,
        /// AVX-512/AVX10 64-byte zmm registers
        Zmm [, #zmm _ 0..32] (64) @ VectorAny overlaps [Xmm, Ymm] = RegisterKind::VectorAny,
        /// AMX Tile Registers
        Tmm [, #tmm _ 0..32] (1024) @ VectorAny = RegisterKind::VectorAny,
        /// AVX-512/AVX10 mask registers
        Kreg [, #k _ 0..32] (8) @ VectorInt = RegisterKind::VectorBit,
        /// Control Registers. These require a privileged (kernel) context to access
        Control [, #cr _ 0..32] @ Custom(RegisterKind::System) = RegisterKind::System,
        /// Debug Registers. These require a privileged (kernel) context to access
        Debug [, #dr _ 0..32] @ Custom(RegisterKind::System) = RegisterKind::System,
        /// Extended (XSAVE) Control Registers. These can be read in any mode (provided the feature is available) but can only be written in privileged code
        ExtControl [, #xcr _ 0..32] (8) @ Custom(RegisterKind::System) = RegisterKind::System,
        /// Synthetic register group for the 3 system registers maintained by x87 (fcw: Floating-point Control Word, fsw: Floating-point Status word, and ftw: Floating-point Tag Word)
        X87SysReg [fcw, fsw, ftw] (2) @ Custom(RegisterKind::System) = RegisterKind::System,
        /// Synthetic register group for the mxcsr register
        SseSysReg [mxcsr] (4) @ Custom(RegisterKind::System) = RegisterKind::System,
    }
}

/// Type that describes the supported sizes of general purpose registers
/// [`GprSize`] can be used to convert general purpose registers between the four possibly [`GprSize`]s or to obtain one of the first 8 GPRs
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GprSize {
    /// Byte (8-bit) register size
    Byte,
    /// Word (16-bit) register size
    Word,
    /// Doubleword (32-bit) register size
    Double,
    /// Quadword (64-bit) register size
    Quad,
}

impl GprSize {
    /// The size, in bytes, of the [`GprSize`]
    pub const fn size(self) -> u32 {
        match self {
            Self::Byte => 1,
            Self::Word => 2,
            Self::Double => 4,
            Self::Quad => 8,
        }
    }

    /// Creates a [`GprSize`] from a valid size
    /// 
    /// ## Panics
    /// Panics if the size is not a power of 2 less than or equal to 8
    pub const fn from_size(val: u32) -> Self {
        match val.next_power_of_two() {
            1 => Self::Byte,
            2 => Self::Word,
            4 => Self::Double,
            8 => Self::Quad,
            _ => panic!("Invalid size class"),
        }
    }

    /// Obtains the width (in bits) of the [`GprSize`].
    pub const fn bits(self) -> u32 {
        match self {
            Self::Byte => 8,
            Self::Word => 16,
            Self::Double => 32,
            Self::Quad => 64,
        }
    }
}

/// Helper enum for creating general purpose registers
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum GprName {
    /// The `rAX` register
    ax,
    /// The `rCX` register
    cx,
    /// The `rDX` register
    dx,
    /// The `rBX` register
    bx,
    /// The `rSP` register
    sp,
    /// The `rBP` register
    bp,
    /// The `rSI` register
    si,
    /// The `rDI` register
    di
}

impl GprName {
    /// Converts to the corresponding [`X86Register`] given the [`GprSize`]
    /// 
    /// [`sp`][GprName::sp], [`bp`][GprName::bp], [`si`][GprName::si], and [`di`][GprName::di] are only accessible as [`GprSize::Byte`] in [`X86Mode::Long`]
    pub const fn as_reg(self, size: GprSize) -> X86Register {
        let regno = self as u8;
        match size {
            GprSize::Byte => {
                if regno < 4 {
                    X86Register::Byte(regno)
                } else {
                    X86Register::ByteRex(regno)
                }
            },
            GprSize::Word => X86Register::Word(regno),
            GprSize::Double => X86Register::Double(regno),
            GprSize::Quad => X86Register::Quad(regno),
        }
    }
}

/// The size of an xmm/ymm/zmm register. Allows converting between registers in these classes
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XmmSize {
    /// The xmm (16-byte) register size
    Xmm,
    ///The ymm (32-byte) register size
    Ymm,
    /// The zmm (64-byte) register size
    Zmm,
}

impl X86Register {
    /// Promotes a general purpose register to the specified one given [`GprSize`]
    /// 
    /// See [`GprName::as_reg`]
    /// 
    /// ## Panics
    /// Panics if `*self` is not a General Purpose Register
    pub const fn promote_gpr(&self, new_size: GprSize) -> X86Register {
        match (self, new_size) {
            (Self::ByteLegacy(n), GprSize::Byte) if *n < 4 => Self::Byte(*n),
            (Self::ByteLegacy(n), GprSize::Word) if *n < 4 => Self::Word(*n),
            (Self::ByteLegacy(n), GprSize::Double) if *n < 4 => Self::Double(*n),
            (Self::ByteLegacy(n), GprSize::Quad) if *n < 4 => Self::Quad(*n),
            (Self::ByteLegacy(n), GprSize::Byte) => Self::ByteLegacy(*n),
            (
                Self::ByteRex(n) | Self::Byte(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n),
                GprSize::Byte,
            ) => {
                if *n < 4 {
                    Self::Byte(*n)
                } else {
                    Self::ByteRex(*n)
                }
            }
            (
                Self::ByteRex(n) | Self::Byte(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n),
                GprSize::Word,
            ) => Self::Word(*n),
            (
                Self::ByteRex(n) | Self::Byte(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n),
                GprSize::Double,
            ) => Self::Double(*n),
            (
                Self::ByteRex(n) | Self::Byte(n) | Self::Word(n) | Self::Double(n) | Self::Quad(n),
                GprSize::Quad,
            ) => Self::Quad(*n),
            _ => panic!("Not a GPR"),
        }
    }

    /// Obtains the [`GprSize`] of the current register, if it is a GPR, or returns [`None`] otherwise.
    pub const fn gpr_size(&self) -> Option<GprSize> {
        match self {
            Self::Byte(_) | Self::ByteRex(_) => Some(GprSize::Byte),
            Self::Word(_) => Some(GprSize::Word),
            Self::Double(_) => Some(GprSize::Double),
            Self::Quad(_) => Some(GprSize::Quad),
            _ => None,
        }
    }
    
    /// Promotes Xmm/Ymm/Zmm registers to the specified [`XmmSize`]. 
    /// 
    /// ## Panics
    /// Panics if `*self` is not an `xmm`, `ymm`, or `zmm` register
    pub const fn promote_xmm(&self, new_size: XmmSize) -> X86Register {
        match (self, new_size) {
            (Self::Xmm(n) | Self::Ymm(n) | Self::Zmm(n), XmmSize::Xmm) => Self::Xmm(*n),
            (Self::Xmm(n) | Self::Ymm(n) | Self::Zmm(n), XmmSize::Ymm) => Self::Ymm(*n),
            (Self::Xmm(n) | Self::Ymm(n) | Self::Zmm(n), XmmSize::Zmm) => Self::Zmm(*n),
            _ => panic!("Not a Vector Register"),
        }
    }

    /// Obtains the [`XmmSize`] of the current register, if it is a xmm/ymm/zmm register, or returns [`None`] otherwise.
    pub const fn xmm_size(&self) -> Option<XmmSize> {
        match self {
            Self::Xmm(_) => Some(XmmSize::Xmm),
            Self::Ymm(_) => Some(XmmSize::Ymm),
            Self::Zmm(_) => Some(XmmSize::Zmm),
            _ => None,
        }
    }

    /// Determines if the `*self` is usable in the specified [`X86Mode`]. 
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
            X86Register::ByteRex(_) => false,
            _ => true,
        }
    }
}

impl const AsId<Register> for X86Register {}

impl RegisterSpec for X86Register {
    type MachineMode = X86Mode;
    fn kind(&self) -> RegisterKind {
        self.kind()
    }

    fn size(&self, mode: X86Mode) -> u32 {
        self.size(mode)
    }

    fn align(&self, mode: Self::MachineMode) -> u32 {
        match self {
            Self::St(_) => 2,
            _ => self.size(mode),
        }
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

    fn category(&self, _: Self::MachineMode) -> crate::xva::XvaCategory {
        self.category()
    }

    fn from_bit(bit: u32, mode: Self::MachineMode) -> Option<Self> {
        match bit {
            n @ 0..0x20 => match mode {
                X86Mode::Real | X86Mode::Protected16 => Some(X86Register::Word(n as u8)),
                X86Mode::Protected => Some(X86Register::Double(n as u8)),
                X86Mode::Long => Some(X86Register::Quad(n as u8)),
            },
            n @ 0x20..0x40 => Some(X86Register::Xmm((n & 31) as u8)),
            n @ 0x40..0x46 => Some(X86Register::Segment((n & 7) as u8)),
            n @ 0x48..0x4C => Some(X86Register::ByteLegacy((n & 7) as u8)),
            n @ 0x50..0x58 => Some(X86Register::Tmm((n & 7) as u8)),
            n @ 0x58..0x60 => Some(X86Register::Kreg((n & 7) as u8)),
            n @ 0x60..0x68 => Some(X86Register::St((n & 7) as u8)),
            n @ 0x68..0x6C => Some(X86Register::X87SysReg((n & 3) as u8)),
            n @ 0x6C..0x6D => Some(X86Register::SseSysReg((n & 3) as u8)),
            _ => None,
        }
    }

    fn regmap_bit(self) -> Option<u32> {
        match self {
            X86Register::ByteLegacy(n @ 4..7) => Some(0x48 | n as u32),
            X86Register::ByteLegacy(n)
            | X86Register::Byte(n)
            | X86Register::ByteRex(n)
            | X86Register::Word(n)
            | X86Register::Double(n)
            | X86Register::Quad(n) => Some(n as u32),
            X86Register::Segment(n) => Some(0x40 | n as u32),
            X86Register::St(n) => Some(0x60 | n as u32),
            X86Register::Mmx(n) => Some(0x60 | ((8 - n) & 7) as u32),
            X86Register::Xmm(n) | X86Register::Ymm(n) | X86Register::Zmm(n) => {
                Some(0x20 | n as u32)
            }
            X86Register::Tmm(n) => Some(0x50 | n as u32),
            X86Register::Kreg(n) => Some(0x58 | n as u32),
            X86Register::Control(_) | X86Register::Debug(_) | X86Register::ExtControl(_) => None,
            X86Register::X87SysReg(n) => Some(0x68 | n as u32),
            X86Register::SseSysReg(n) => Some(0x6C | n as u32),
        }
    }
}

/// Expands to a constant array of [`X86Register`]s with the specified name
/// Fails to compile if any register is invalid
#[macro_export]
macro_rules! x86_registers {
    [$($reg:ident),* $(,)?] => {
        const { [$($crate::x86_register!($reg)),*]}
    }
}

/// Expands to a constant [`X86Register`] with the specified name.
/// Fails to compile if the register is invalid
#[macro_export]
macro_rules! x86_register {
    [$reg:ident] => {
        const { $crate::archs::x86::X86Register::from_name_impl(::core::stringify!($reg))}
    }
}

/// Enum for the kind of opcodes. Useful for Encoding

pub enum X86OperandKind {
    /// Register operand with the specified class
    Register(X86RegisterClass),
    /// Immediate operand
    Immediate,
    /// Memory operand with the specified data size
    Memory(X86RegisterClass),
    /// Relative Address
    RelAddr,
}

macro_rules! x86_instructions {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(!prefix $(#[$prefix_meta:meta])* $prefix_name:ident ($prefix_mnemonic:literal) = $prefix_opcode:literal;)*
            $($(#[$instr_meta:meta])*  $instr_name:ident ($mnemonic:literal) {
                $([$($frag:tt @ $operand:pat),* $(,)?] $($mode:pat)? => $opcode:literal $(+ $regno:expr)?),+ $(,)?
            })*
        }
    } => {

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId)]
        $(#[$meta])*
        $vis enum $name {
            $(
                #[doc = ::core::concat!("The `", $prefix_mnemonic, "` prefix")]    
                $(#[$prefix_meta])* 
                $prefix_name,
            )*
            $(
                #[doc = ::core::concat!("The `", $mnemonic, "` instruction")]  
                $(#[$instr_meta])* 
                $instr_name
            ),*
        }

        impl $name {
            const ALL_OPCODES: [Self; ${count($instr_name)} + ${count($prefix_name)}] = [$(Self::$prefix_name,)* $(Self::$instr_name),*];
        }

        impl const $crate::traits::AsId<$crate::mach::Opcode> for $name {}

        impl $crate::traits::Name for $name {
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$prefix_name => $prefix_mnemonic,)*
                    $(Self::$instr_name => $mnemonic),*
                }
            }
        }

    };
}

x86_instructions! {
    /// All X86 Opcodes.
    pub enum X86Opcode {
        !prefix
        Lock ("lock") = 0xF0;
        !prefix
        AddrOverride ("addro") = 0x67;
        !prefix
        DataOverride ("datao") = 0x66;
        !prefix
        Repnz ("repnz") = 0xF2;
        !prefix
        Repz ("repz") = 0xF3;
        !prefix
        Rep ("rep") = 0xF3;
        !prefix
        Wait ("fwait") = 0x9B;
        Add ("add") {
            [_ @ Memory(X86RegisterClass::Byte) | Register(X86RegisterClass::Byte), _ @ Register(X86RegisterClass::Byte)] => 0x00,
        }
        Sub ("sub") {
            [_ @ Memory(X86RegisterClass::Byte) | Register(X86RegisterClass::Byte), _ @ Register(X86RegisterClass::Byte)] => 0x28,
        }
        Or ("sub") {
            [_ @ Memory(X86RegisterClass::Byte) | Register(X86RegisterClass::Byte), _ @ Register(X86RegisterClass::Byte)] => 0x08,
        }
        And ("and") {
            [_ @ Memory(X86RegisterClass::Byte) | Register(X86RegisterClass::Byte), _ @ Register(X86RegisterClass::Byte)] => 0x20,
        }
        Xor ("xor") {
            [_ @ Memory(X86RegisterClass::Byte) | Register(X86RegisterClass::Byte), _ @ Register(X86RegisterClass::Byte)] => 0x30,
        }
        Mov ("mov") {
            [_ @ Memory(X86RegisterClass::Byte) | Register(X86RegisterClass::Byte), _ @ Register(X86RegisterClass::Byte)] => 0x88,
        }
        Lea ("lea") {
            [_ @ Register(X86RegisterClass::Word | X86RegisterClass::Double | X86RegisterClass::Quad), _ @  Memory(_)] => 0x8D,
        }
        Call ("call") {
            [_ @ RelAddr] => 0xE8,
        }
        Jump ("jmp") {
            [_ @ RelAddr] => 0xE9,
        }

        Ud2 ("ud2") {
            [] => 0x0F0B
        }
        Push ("push") {
            [dest @ Register(X86RegisterClass::Word | X86RegisterClass::Double | X86RegisterClass::Quad)] => 0x50 + (dest.regno() & 7),
        }
        Pop ("pop") {
            [dest @ Register(X86RegisterClass::Word | X86RegisterClass::Double | X86RegisterClass::Quad)] => 0x58 + (dest.regno() & 7),
        }
        Ret ("ret") {
            [] => 0xC3,
            [_ @ Immediate] => 0xC2
        }
    }
}

/// [`Machine`][crate::mach::Machine] and [`Compiler`][crate::compiler::Compiler] for x86
pub struct X86;

impl MachineSpec for X86 {
    type MachineMode = X86Mode;
    type Compiler = Self;
    type Opcode = X86Opcode;
    type Register = X86Register;

    const MACH_MODES: &[MachineMode] = as_id_array!(X86Mode::ALL_MODES => MachineMode);
    const REGISTERS: &[Register] = as_id_array!(X86Register::ALL_REGISTERS => Register);
    const OPCODES: &[Opcode] = as_id_array!(X86Opcode::ALL_OPCODES => Opcode);

    fn as_compiler(&self) -> &Self::Compiler {
        self
    }

    fn name(&self) -> &'static str {
        "x86"
    }

    fn pretty_print_size(&self, size: usize) -> Option<&'static str> {
        match size {
            1 => Some("byte"),
            2 => Some("word"),
            4 => Some("dword"),
            8 => Some("qword"),
            10 => Some("tbyte"),
            16 => Some("xmmword"),
            32 => Some("ymmword"),
            64 => Some("zmmword"),
            1024 => Some("tmmword"),
            _ => None,
        }
    }
}

impl X86 {
    fn opcode_for_expr(&self, dest: X86Register, dest2: Option<X86Register>, expr: &XvaOpcode) -> Option<X86Opcode>{
        match expr {
            XvaOpcode::ZeroInit => {
                match dest {
                    X86Register::Byte(_) |
                    X86Register::ByteLegacy(_) |
                    X86Register::ByteRex(_) |
                    X86Register::Word(_) |
                    X86Register::Double(_) |
                    X86Register::Quad(_) => {
                        Some(X86Opcode::Xor)
                    },
                    
                    X86Register::Mmx(_) => todo!(),
                    X86Register::Xmm(_) => todo!(),
                    X86Register::Ymm(_) => todo!(),
                    X86Register::Zmm(_) => todo!(),
                    X86Register::Tmm(_) => todo!(),
                    X86Register::Kreg(_) => todo!(),
                    X86Register::St(_) => todo!(),
                    X86Register::Segment(_) |
                    X86Register::Control(_) |
                    X86Register::Debug(_) |
                    X86Register::ExtControl(_) |
                    X86Register::X87SysReg(_) |
                    X86Register::SseSysReg(_)
                    => panic!("Cannot support zeroinit of these registers"),
                }
            },
            
            XvaOpcode::Uninit => None,
            XvaOpcode::Const(val) => {
                match val {
                    crate::xva::XvaConst::Bits(_) => Some(X86Opcode::Mov),
                    crate::xva::XvaConst::Label(_) |
                    crate::xva::XvaConst::Global(_, _) => Some(X86Opcode::Lea),
                }
            },
            XvaOpcode::Move(src) => {
                let XvaRegister::Physical(preg) = *src else {
                    panic!("Encountered virtual register in preg")
                };

                match dest {
                    X86Register::Byte(_) |
                    X86Register::ByteLegacy(_) |
                    X86Register::ByteRex(_) |
                    X86Register::Word(_) |
                    X86Register::Double(_) |
                    X86Register::Quad(_) |
                    X86Register::Control(_) |
                    X86Register::Debug(_) |
                    X86Register::Segment(_) => Some(X86Opcode::Mov),
                    X86Register::St(_) => todo!("st"),
                    X86Register::Mmx(_) => todo!("mmx"),
                    X86Register::Xmm(_) => todo!("xmm"),
                    X86Register::Ymm(_) => todo!("ymm"),
                    X86Register::Zmm(_) => todo!("zmm"),
                    X86Register::Tmm(_) => todo!("tmm"),
                    X86Register::Kreg(_) => todo!("kreg"),
                    X86Register::ExtControl(_) => todo!("xcr"),
                    X86Register::X87SysReg(_) | X86Register::SseSysReg(_) => panic!("Cannot move to a fsw/fcw/ftw/mxcsr (need to use read)"),
                }
            },
            XvaOpcode::ComputeAddr { base, size, index } => todo!(),
            XvaOpcode::GetFrameAddr(_) => todo!(),
            XvaOpcode::BinaryOp { op, left, right } => {
                match (*op, dest) {
                    (BinaryOp::Add, X86Register::Byte(_) |
                        X86Register::ByteLegacy(_) |
                        X86Register::ByteRex(_) |
                        X86Register::Word(_) |
                        X86Register::Double(_) |
                        X86Register::Quad(_)
                    ) => {
                        Some(X86Opcode::Add)
                    }
                    (BinaryOp::Sub, X86Register::Byte(_) |
                        X86Register::ByteLegacy(_) |
                        X86Register::ByteRex(_) |
                        X86Register::Word(_) |
                        X86Register::Double(_) |
                        X86Register::Quad(_)
                    ) => {
                        Some(X86Opcode::Sub)
                    }
                    (BinaryOp::And, X86Register::Byte(_) |
                        X86Register::ByteLegacy(_) |
                        X86Register::ByteRex(_) |
                        X86Register::Word(_) |
                        X86Register::Double(_) |
                        X86Register::Quad(_)
                    ) => {
                        Some(X86Opcode::And)
                    }
                    (BinaryOp::Or, X86Register::Byte(_) |
                        X86Register::ByteLegacy(_) |
                        X86Register::ByteRex(_) |
                        X86Register::Word(_) |
                        X86Register::Double(_) |
                        X86Register::Quad(_)
                    ) => {
                        Some(X86Opcode::Or)
                    }
                    (BinaryOp::Xor, X86Register::Byte(_) |
                        X86Register::ByteLegacy(_) |
                        X86Register::ByteRex(_) |
                        X86Register::Word(_) |
                        X86Register::Double(_) |
                        X86Register::Quad(_)
                    ) => {
                        Some(X86Opcode::Xor)
                    }
                    _ => todo!("Combination")
                }
            },
            XvaOpcode::CheckedBinaryOp { op, mode, left, right } => todo!(),
            XvaOpcode::UnaryOp { op, left } => todo!(),
            XvaOpcode::Read(xva_operand) => todo!(),
            XvaOpcode::UMul { left, right } => todo!(),
            XvaOpcode::SMul { left, right } => todo!(),
        }
    }
}

impl CompilerSpec for X86 {
    type Machine = Self;

    fn available_registers(
        &self,
        context: &crate::compiler::CompilerContext,
        mode: Self::MachineMode,
        cat: crate::xva::XvaCategory,
        size: u32,
    ) -> Option<&[Register]> {
        None
    }

    fn promote_size(
        &self,
        context: &crate::compiler::CompilerContext,
        mode: X86Mode,
        cat: XvaCategory,
        size: u32,
    ) -> Option<u32> {
        match (cat, size) {
            (XvaCategory::Int, size @ ..=8) => {
                if size > mode.largest_gpr().size() {
                    None
                } else {
                    Some(size.next_power_of_two())
                }
            }
            (XvaCategory::Float, size @ (4 | 8)) => {
                if context.target_features.contains("sse") {
                    Some(size)
                } else {
                    Some(10)
                }
            }
            (XvaCategory::Float, 10) => Some(10),
            (XvaCategory::Float, size) => Some(size.next_power_of_two()), // Must be promoted to an sse register, which may error
            (
                XvaCategory::VectorAny | XvaCategory::VectorFloat | XvaCategory::VectorInt,
                size @ ..=64,
            ) => Some(size.next_power_of_two()),
            (XvaCategory::Condition, 1) => Some(1),
            _ => None,
        }
    }

    fn lower_mce(&self, stmt: &mut XvaStatement, mode: X86Mode) {
        let instr = match stmt {
            XvaStatement::Expr(xva_expr) => {
                let XvaRegister::Physical(dest) = xva_expr.dest else {
                    panic!("Virtual Register during mce")
                };

                let dest = dest.downcast::<X86Register>().expect("Non-x86 register encountered");
                let dest2 = xva_expr.dest2.map(|v| {
                    let XvaRegister::Physical(v2) = v else {
                        panic!("Virtual Register during mce")
                    };

                    v2.downcast::<X86Register>().expect("Non x86-register encountered")
                });

                let Some(opcode) = self.opcode_for_expr(dest, dest2, &xva_expr.op) else {
                    *stmt = XvaStatement::Elaborated(vec![]); 
                    return;
                };

                let mut oprs = Vec::with_capacity(2);
                oprs.push(Operand::Register(Register::new(dest)));

                match &xva_expr.op {
                    XvaOpcode::ZeroInit | XvaOpcode::Uninit => {
                        oprs.push(Operand::Register(Register::new(dest)));
                    },
                    XvaOpcode::Const(xva_const) => {
                        oprs.push(xva_const.to_readable(AddressKind::Default, AddressKind::GotRel, mode.supports_rel_addr(), None));
                    },
                    
                    XvaOpcode::Move(reg) => {
                        let XvaRegister::Physical(reg) = *reg else {
                            panic!("Virtual Register during mce")
                        };

                        oprs.push(Operand::Register(reg))
                    },
                    XvaOpcode::ComputeAddr { base, size, index } => todo!(),
                    XvaOpcode::GetFrameAddr(_) => todo!(),
                    XvaOpcode::BinaryOp { op, left, right } => {
                        let XvaRegister::Physical(left) = *left else {
                            panic!("Virtual Register during mce")
                        };

                        if left != Register::new(dest) {
                            oprs.push(Operand::Register(left));
                        }

                        let size = dest.size(mode);

                        match right {
                            crate::xva::XvaOperand::Register(reg) => {
                                let XvaRegister::Physical(reg) = *reg else {
                                    panic!("Virtual Register during mce")
                                };
                                oprs.push(Operand::Register(reg));
                            },
                            crate::xva::XvaOperand::Const(xva_const) => oprs.push(xva_const.to_readable(AddressKind::Default, AddressKind::Plt, mode.supports_rel_addr(), Some(size as usize))),
                            crate::xva::XvaOperand::FrameAddr(_) => todo!(),
                        }
                    },
                    XvaOpcode::CheckedBinaryOp { op, mode, left, right } => todo!(),
                    XvaOpcode::UnaryOp { op, left } => todo!(),
                    XvaOpcode::Read(xva_operand) => todo!(),
                    XvaOpcode::UMul { left, right } => todo!(),
                    XvaOpcode::SMul { left, right } => todo!(),
                }

                Instruction::new(Opcode::new(opcode), oprs)
            },
            XvaStatement::Write(xva_operand, xva_register) => todo!("write"),
            XvaStatement::Jump(symbol) => {
                Instruction::new(Opcode::new(X86Opcode::Jump), vec![Operand::RelSymbol(RelocSym { sym: *symbol, kind: AddressKind::Default }, None)])
            },
            XvaStatement::Tailcall { dest, .. } => {
                let mut oprs = Vec::with_capacity(1);
                match *dest {
                    crate::xva::XvaOperand::Register(reg) => {
                        let XvaRegister::Physical(reg) = reg else {
                            panic!("Virtual Register during mce")
                        };
                        oprs.push(Operand::Register(reg))
                    },
                    crate::xva::XvaOperand::Const(xva_const) => {
                        oprs.push(xva_const.to_direct_rel(AddressKind::Default, AddressKind::Plt));
                    },
                    crate::xva::XvaOperand::FrameAddr(_) => unreachable!("Cannot call the stack"),
                }
                Instruction::new(Opcode::new(X86Opcode::Jump), oprs)
            },
            XvaStatement::Call { dest, .. } => {
                let mut oprs = Vec::with_capacity(1);
                match *dest {
                    crate::xva::XvaOperand::Register(reg) => {
                        let XvaRegister::Physical(reg) = reg else {
                            panic!("Virtual Register during mce")
                        };
                        oprs.push(Operand::Register(reg))
                    },
                    crate::xva::XvaOperand::Const(xva_const) => {
                        oprs.push(xva_const.to_direct_rel(AddressKind::Default, AddressKind::Plt));
                    },
                    crate::xva::XvaOperand::FrameAddr(_) => unreachable!("Cannot call the stack"),
                }

                Instruction::new(Opcode::new(X86Opcode::Call), oprs)
            },
            XvaStatement::Return => {
                Instruction::new_nullary(X86Opcode::Ret)
            }
            XvaStatement::Trap(_) => Instruction::new_nullary(X86Opcode::Ud2),
            XvaStatement::Noop(_) => todo!("special noop"),

            _ => unreachable!()
        };

        *stmt = XvaStatement::RawInstr(instr);
    }

    fn lower_epilogue(&self, frame: crate::xva::XvaFrameProperties, mode: X86Mode) -> Vec<XvaStatement> {
        let mode_gpr = mode.largest_gpr();
        let sp = GprName::sp.as_reg(mode_gpr);
        let mut epilogue = Vec::new();
        if frame.use_frame_pointer {
            let bp = GprName::bp.as_reg(mode_gpr);
            epilogue.push(XvaStatement::RawInstr(Instruction::new(Opcode::new(X86Opcode::Mov), vec![Operand::Register(Register::new(sp)), Operand::Register(Register::new(bp))])));
            epilogue.push(XvaStatement::RawInstr(Instruction::new(Opcode::new(X86Opcode::Pop), vec![Operand::Register(Register::new(bp))])));
        } else if frame.has_prologue {
            let size = frame.frame_size;
            epilogue.push(XvaStatement::RawInstr(Instruction::new(Opcode::new(X86Opcode::Add), vec![Operand::Register(Register::new(sp)), Operand::Immediate(size as u128)])));
        }
        epilogue
    }

    fn emit_prologue(&self, frame: &mut crate::xva::XvaFrameProperties, mode: X86Mode) -> Vec<Instruction> {
        let mode_gpr = mode.largest_gpr();
        let sp = GprName::sp.as_reg(mode_gpr);
        
        let mut used_size = 0;
        let mut align_frame = false;
        if frame.call_align < frame.frame_align {
            frame.use_frame_pointer = true;
            align_frame = true;
        }
        let mut instrs = Vec::new();
        if frame.use_frame_pointer {
            let bp = GprName::bp.as_reg(mode_gpr);
            let fptr_size = mode_gpr.size() as usize;
            frame.frame_size += fptr_size;
            used_size += 8;
            instrs.push(Instruction::new(Opcode::new(X86Opcode::Push), vec![Operand::Register(Register::new(bp))]));
            instrs.push(Instruction::new(Opcode::new(X86Opcode::Mov), vec![Operand::Register(Register::new(bp)), Operand::Register(Register::new(sp))]));
        }

        let mut align_offset = frame.call_align_offset;

        if align_frame {
            let align = !(frame.frame_align - 1) as u32;
            instrs.push(Instruction::new(Opcode::new(X86Opcode::And), vec![Operand::Register(Register::new(sp)), Operand::Immediate(align as u128)]));

            align_offset = 0;
        }
        let total_size = frame.frame_size + align_offset;

        let disp = total_size & (frame.frame_align - 1);

        if disp != 0 {
            frame.frame_size += frame.frame_align - disp;
        }

        let sub_size = frame.frame_size - used_size;

        if sub_size > 0 {
            instrs.push(Instruction::new(Opcode::new(X86Opcode::Sub), vec![Operand::Register(Register::new(sp)), Operand::Immediate(sub_size as u128)]));
        }
        
        frame.has_prologue = !instrs.is_empty();

        instrs
    }
}
