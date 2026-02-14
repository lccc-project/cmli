use std::num::NonZeroI64;

use crate::{
    intern::Symbol,
    mach::{MachineMode, Opcode, Register},
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Operand {
    Register(Register),
    Immediate(u128),
    AbsAddress(Address),
    RelAddress(Address),
    Memory(MemoryOperand),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MemoryOperand {
    pub value_size: Option<usize>,
    pub addr: Address,
    pub rel: bool,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Address {
    pub segment: Option<Register>,
    pub base: Option<Register>,
    pub index: Option<Register>,
    pub scale: u32,
    pub sym: Option<Symbol>,
    pub disp: Option<NonZeroI64>,
}
