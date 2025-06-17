use std::num::NonZeroU64;

use crate::{
    error::Result,
    instr::{Instruction, RegisterClassId, RegisterId},
    intern::Symbol,
    xva::XvaInstr,
};

pub trait Compiler {
    fn allocate_registers(&self, xva: &mut [XvaInstr]) -> Result<()>;
    fn lower_to_mc(&self, xva: &[XvaInstr], mc: &mut Vec<MceInstr>) -> Result<()>;
    fn legalize_mc(&self, mc: &mut [MceInstr]) -> Result<()> {
        Ok(())
    }
    fn optimize_mc(&self, mc: &mut Vec<MceInstr>) -> Result<()> {
        Ok(())
    }
}
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PsuedoReg {
    Physical(RegisterId),
    Virtual(EmphemeralRegister),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct EmphemeralRegister(NonZeroU64);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterType {
    Integer { size: u16 },
    Float { size: u16 },
    Vector { elem_width: u8, lane_count: u8 },
    ArchSpecific(RegisterClassId),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MceInstr {
    Empty,
    Single(Instruction),
    Multiple(Vec<Instruction>),
    StopAnalysis,
    StartAnalysis,
    LabelMarker(Symbol),
}
