#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Width {
    Bits(u16),
    Native,
}

use Width::*;

include!(env!("CMLI_DEF_x86"));
