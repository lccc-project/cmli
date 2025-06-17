use std::{num::NonZeroU64, path::Prefix};

use indexmap::IndexMap;

use crate::{
    asm::{Parser, Printer},
    compiler::Compiler,
    encode::{Decoder, Encoder},
    helpers::def_id_type,
    intern::Symbol,
    traits::{IdentityName, IntoId, Unique, into_id},
};

pub trait Opcodes: Unique {
    fn encoding_info(&self) -> &OpcodesEncoding;
    fn instr_spec(&self, id: InstructionId) -> crate::Result<InstructionSpec>;
    fn prefix_spec(&self, id: PrefixId) -> crate::Result<PrefixSpec>;
    fn legalize<'a>(&'a self, instr: &Instruction) -> Vec<Instruction>;
    fn default_encoding_mode(&self) -> EncodingId;
    fn make_parser(&self) -> Option<Box<dyn Parser + '_>>;
    fn make_printer(&self) -> Box<dyn Printer + '_>;
    fn make_encoder(&self) -> Box<dyn Encoder + '_>;
    fn make_decoder(&self) -> Option<Box<dyn Decoder + '_>>;
    fn make_compiler(&self) -> Box<dyn Compiler + '_>;
    fn reg_class(&self, reg: RegisterId) -> RegisterClassId;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct OpcodesEncoding {
    pub min_op_bytes: u16,
    pub max_op_bytes: u16,
    pub op_word_size: u16,
    pub op_word_alignment: u16,

    #[doc(hidden)]
    pub __non_exhaustive: (),
}

def_id_type!(InstructionId);
def_id_type!(InstructionFieldId);
def_id_type!(PrefixId);
def_id_type!(EncodingId);
def_id_type!(RegisterId);
def_id_type!(RegisterClassId);
def_id_type!(ExtendedConditionId);

def_id_type!(RelocTypeId);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, InstructionFieldId)]
pub enum NoField {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstructionSpec<'a> {
    pub name: &'a str,
    pub id: InstructionId,
    pub fields: &'a [InstructionFieldId],
    pub valid_prefixes: &'a [PrefixId],
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PrefixSpec<'a> {
    pub name: &'a str,
    pub id: PrefixId,
    pub fields: &'a [InstructionFieldId],
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Operand {
    Immediate(u64),
    Address(Address),
    Memory(Address),
    Register(RegisterId),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RelocationType {
    Normal,
    Got,
    Plt,
    Tpoff,
    DynTpOff,
    TlsGd,
    TlsLd,
    Reloc(RelocTypeId),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SymbolSpec {
    pub sym: Symbol,
    pub reloc_ty: RelocationType,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Offset {
    pub sym: Option<SymbolSpec>,
    pub disp: i64,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Address {
    Abs(Offset),
    Rel(Offset),
    Complex(ComplexAddress),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ComplexAddress {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum InstructionFieldValue {
    Flag(bool),
    Val(u64),
    Register(RegisterId),
    ConditionCode(ConditionCode),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ConditionCode {
    Equal,
    NotEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    encoding_override: EncodingId,
    spec: InstructionId,
    fields: IndexMap<InstructionFieldId, InstructionFieldValue>,
    operands: Vec<Operand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefix {
    spec: PrefixId,
    fields: IndexMap<InstructionFieldId, InstructionFieldValue>,
}

#[doc(hidden)]
pub mod macros;
