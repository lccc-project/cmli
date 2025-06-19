use crate::{helpers::def_id_type, instr::Instruction};

def_id_type!(RelocId);

pub trait InstrWrite: std::io::Write {}

pub trait InstrRead: std::io::Read {}

pub trait Encoder {
    fn encode_instr(
        &self,
        writer: &mut (dyn InstrWrite + '_),
        instr: Instruction,
    ) -> crate::Result<()>;
}

pub trait Decoder {
    fn decode_instr(&self, read: &mut (dyn InstrRead + '_)) -> crate::Result<Instruction>;
}
