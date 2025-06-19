use crate::instr_set;

pub enum Width {
    Bits(u16),
    Native,
}

use Width::*;

include!(env!("CMLI_DEF_x86"));
