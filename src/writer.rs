
use std::io::{Write, Result};

use crate::{instr::Instruction, reloc::RelocValue};

pub trait RelocatableWriter : Write {
    fn write_with_reloc(&mut self, data: &[u8], reloc: RelocValue) -> Result<()>;
}

pub trait Encoder {
    fn encode_instr(&self, writer: &mut dyn RelocatableWriter, instr: Instruction) -> Result<()>;
}
