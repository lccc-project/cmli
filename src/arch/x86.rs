use crate::instr::{EncodingId, InstructionId};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, InstructionId)]
pub enum X86Opcode {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, EncodingId)]
pub enum X86Mode {
    Real,
    Protected,
    Long,
}
