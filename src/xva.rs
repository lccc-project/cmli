use crate::{
    compiler::{PseudoReg, RegisterType},
    instr::InstructionId,
    intern::Symbol,
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Order {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Opcode {
    ZeroInit,
    NoInit,
    ConstScalar(u128),
    ConstAddr(Symbol, u64),
    Copy(PseudoReg),
    LoadAddr(Symbol, u64, Order),
    Load(PseudoReg, Order),
    StoreAddr(Symbol, u64, Order),
    Store(PseudoReg, Order),
    AddSigned(PseudoReg, PseudoReg),
    AddUnsigned(PseudoReg, PseudoReg),
    SubSigned(PseudoReg, PseudoReg),
    SubUnsigned(PseudoReg, PseudoReg),
    MulSigned(PseudoReg, PseudoReg),

    Native(InstructionId, Option<PseudoReg>, Option<PseudoReg>),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaInstr {
    pub dest: PseudoReg,
    pub aux: Option<PseudoReg>,
    pub ty: RegisterType,
    pub op: Opcode,
}
