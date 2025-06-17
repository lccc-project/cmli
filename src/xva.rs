use crate::{
    compiler::{PsuedoReg, RegisterType},
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
    Copy(PsuedoReg),
    LoadAddr(Symbol, u64, Order),
    Load(PsuedoReg, Order),
    StoreAddr(Symbol, u64, Order),
    Store(PsuedoReg, Order),
    AddSigned(PsuedoReg, PsuedoReg),
    AddUnsigned(PsuedoReg, PsuedoReg),
    SubSigned(PsuedoReg, PsuedoReg),
    SubUnsigned(PsuedoReg, PsuedoReg),
    MulSigned(PsuedoReg, PsuedoReg),

    Native(InstructionId, Option<PsuedoReg>, Option<PseudoReg>),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaInstr {
    pub dest: PsuedoReg,
    pub aux: Option<PsuedoReg>,
    pub ty: RegisterType,
    pub op: Opcode,
}
