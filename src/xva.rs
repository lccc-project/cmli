use crate::{instr::RegisterKind, intern::Symbol, mach::Register};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaTrap {
    Unreachable,
    Abort,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaStatement {
    Label(Symbol),
    Expr(XvaExpr),
    Write(XvaOperand, XvaRegister),
    Jump(Symbol),
    Tailcall {
        dest: XvaOperand,
        params: Vec<XvaRegister>,
    },
    Return,
    Trap(XvaTrap),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaExpr {
    pub dest: XvaRegister,
    pub dest2: Option<XvaRegister>,
    pub op: XvaOpcode,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaConst {
    Bits(u64),
    Label(Symbol),
    Global(Symbol, i64),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaOperand {
    Register(XvaRegister),
    Const(XvaConst),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaOpcode {
    ZeroInit,
    Const(XvaConst),
    Uninit,
    Alloca(XvaType),
    ComputeAddr {
        base: XvaOperand,
        size: u32,
        index: XvaOperand,
    },
    BinaryOp {
        op: BinaryOp,
        left: XvaRegister,
        right: XvaOperand,
    },
    CheckedBinaryOp {
        op: BinaryOp,
        mode: CheckMode,
        left: XvaRegister,
        right: XvaOperand,
    },
    UnaryOp {
        op: UnaryOp,
        left: XvaRegister,
    },
    Call {
        dest: XvaOperand,
        params: Vec<XvaRegister>,
    },
    Read(XvaOperand),
    UMul {
        left: XvaRegister,
        right: XvaRegister,
    },
    SMul {
        left: XvaRegister,
        right: XvaRegister,
    },
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    ShiftLeft(ShiftBehaviour),
    ShiftRight(ShiftBehaviour),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CheckMode {
    CheckSignedOverflow,
    CheckUnsignedOverflow,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ShiftBehaviour {
    AssumeQuantity,
    WrapQuantity,
    UnboundQuantity,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaType {
    pub size: u32,
    pub align: u32,
    pub category: XvaCategory,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaCategory {
    Null,
    Condition,
    Int,
    Float,
    VectorInt,
    VectorFloat,
    Aggregate,
    Custom(RegisterKind),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaRegister {
    Physical(Register),
    Virtual(XvaDest),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaDest {
    pub id: u32,
    pub ty: XvaType,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaFunction {
    pub params: Vec<XvaRegister>,
    pub return_reg: XvaRegister,
    pub statement: Vec<XvaStatement>,
}
