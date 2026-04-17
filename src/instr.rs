use std::num::{NonZeroI64, NonZeroU32};

use crate::{
    fmt::{PrettyPrinter, pretty_print_list},
    intern::Symbol,
    mach::{MachineMode, Opcode, Register}, traits::{AsId, IdType},
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterKind {
    GeneralPurpose,
    IntegerOnly,
    AddressOnly,
    ScalarFp,
    VectorAny,
    VectorInt,
    VectorFloat,
    VectorBit,
    System,
    ConditionCode,
    Special,
    AddressSegment,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Instruction {
    mode_override: Option<MachineMode>,
    prefixes: Vec<Opcode>,
    backing: Opcode,
    operands: Vec<Operand>,
}

impl Instruction {
    pub const fn new(opcode: Opcode, operands: Vec<Operand>) -> Self {
        Self {mode_override: None, prefixes: vec![], backing: opcode, operands}
    }

    pub const fn new_nullary<O: const AsId<Opcode>>(op: O) -> Self {
        Self::new(Opcode::new(op), vec![])
    }
    pub fn mode_override(&self) -> Option<MachineMode> {
        self.mode_override
    }

    pub fn prefixes(&self) -> &[Opcode] {
        &self.prefixes
    }

    pub fn opcode(&self) -> Opcode {
        self.backing
    }

    pub fn operands(&self) -> &[Operand] {
        &self.operands
    }
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(mach) = self.0.mode_override {
            f.write_str(".mode ")?;
            f.write_str(self.1.modes().name_of(mach))?;
            f.write_str(" ")?;
        }

        pretty_print_list(
            self.0
                .prefixes()
                .iter()
                .chain(core::iter::once(&self.0.backing)),
            " ",
            self.1,
            self.2,
        )
        .fmt(f)?;

        f.write_str(" ")?;
        pretty_print_list(self.0.operands(), ", ", self.1, self.2).fmt(f)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Operand {
    Register(Register),
    Immediate(u128),
    AbsSymbol(RelocSym, Option<NonZeroI64>),
    RelSymbol(RelocSym, Option<NonZeroI64>),
    Memory(MemoryOperand),
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, Operand> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Operand::Register(register) => PrettyPrinter(register, self.1, self.2).fmt(f),
            Operand::Immediate(v) => v.fmt(f),
            Operand::AbsSymbol(address, disp) => {
                f.write_str("abs ")?;
                address.fmt(f)?;
                if let Some(val) = disp {
                    if val.get() < 0 {
                        let val = val.get().unsigned_abs();
                        f.write_fmt(format_args!(" - {val}"))?;
                    } else {
                        f.write_fmt(format_args!(" + {val}"))?;
                    }
                }

                Ok(())
            }
            Operand::RelSymbol(address, disp) => {
                f.write_str("rel ")?;
                address.fmt(f)?;
                if let Some(val) = disp {
                    if val.get() < 0 {
                        let val = val.get().unsigned_abs();
                        f.write_fmt(format_args!(" - {val}"))?;
                    } else {
                        f.write_fmt(format_args!(" + {val}"))?;
                    }
                }

                Ok(())
            }
            Operand::Memory(memory_operand) => PrettyPrinter(memory_operand, self.1, self.2).fmt(f),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MemoryOperand {
    pub value_size: Option<usize>,
    pub addr: Address,
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, MemoryOperand> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value_size) = self.0.value_size {
            if let Some(name) = self.1.pretty_print_size(value_size) {
                f.write_fmt(format_args!("{name} "))?;
            } else {
                f.write_fmt(format_args!("{value_size} bytes "))?;
            }
        }

        f.write_str("[")?;

        PrettyPrinter(&self.0.addr, self.1, self.2).fmt(f)?;
        f.write_str("]")
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Address {
    pub segment: Option<Register>,
    pub base: Option<Register>,
    pub index: Option<Register>,
    pub scale: NonZeroU32,
    pub sym: Option<RelocSym>,
    pub disp: Option<NonZeroI64>,
    pub rel: bool,
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, Address> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.rel {
            f.write_str("rel ")?;
        }
        if let Some(seg) = self.0.segment {
            PrettyPrinter(&seg, self.1, self.2).fmt(f)?;
            f.write_str(":")?;
        }
        let mut sep = "";

        if let Some(sym) = self.0.sym {
            sym.fmt(f)?;
            sep = " + ";
        }

        if let Some(base) = self.0.base {
            f.write_str(sep)?;
            PrettyPrinter(&base, self.1, self.2).fmt(f)?;
            sep = " + ";
        }

        if let Some(index) = self.0.index {
            f.write_str(sep)?;
            sep = " + ";
            if let scale @ 2.. = self.0.scale.get() {
                f.write_fmt(format_args!("{scale}*"))?;
            }
            PrettyPrinter(&index, self.1, self.2).fmt(f)?;
        }

        if let Some(disp) = self.0.disp {
            f.write_str(sep)?;
            sep = " + ";
            disp.fmt(f)?;
        }

        if sep.is_empty() {
            f.write_str("0")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RelocSym {
    pub sym: Symbol,
    pub kind: AddressKind,
}

impl core::fmt::Display for RelocSym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.sym)?;

        match self.kind {
            AddressKind::Default => Ok(()),
            AddressKind::GotRel => f.write_str("@gotpcrel"),
            AddressKind::GotAbs => f.write_str("@gotabs"),
            AddressKind::Plt => f.write_str("@plt"),
            AddressKind::Tpoff => f.write_str("@tpoff"),
            AddressKind::DTpoff => f.write_str("@gottpoff"),
            AddressKind::TlsDesc => f.write_str("@tlsdesc"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum AddressKind {
    Default,
    GotRel,
    GotAbs,
    Plt,
    Tpoff,
    DTpoff,
    TlsDesc,
}
