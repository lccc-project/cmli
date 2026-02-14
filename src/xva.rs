use crate::{
    instr::{Instruction, RegisterKind},
    intern::Symbol,
    mach::{Machine, MachineMode, Register},
    traits::{AsId, IdType as _},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaTrap {
    Unreachable,
    Abort,
}

bitflags::bitflags! {
    /// Indicates the kinds of operations that cannot pass an optimization gate
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct BarrierKind : u32 {
        /// Disallows propagating constant values passed the optimization gate
        const PROPAGATE_THROUGH = 0x0000_0001;
        /// Prevents eliding any instructions at all
        const ELIDE_INSTRS = 0x0000_0002;
        /// Prevents eliding the use of registers
        const ELIDE_REGISTERS = 0x0000_0004;
        /// Prevents eliding stores
        const ELIDE_STORE = 0x0000_0008;
        /// Forbids all Optimizations until the barrier is ended
        const DO_NOT_OPTIMIZE = 0x0000_0010;
    }
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
    Call {
        dest: XvaOperand,
        params: Vec<XvaRegister>,
        ret_val: Vec<XvaRegister>,
        call_clobber_regs: Vec<XvaRegister>,
    },
    Return,
    Trap(XvaTrap),
    RawInstr(Instruction),
    OptGate(BarrierKind, u32),
    EndOptGate(u32),
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
    Move(XvaRegister),
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
    pub size: u64,
    pub align: u64,
    pub category: XvaCategory,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaCategory {
    Null,
    Condition,
    Int,
    Float,
    VectorAny,
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

impl XvaRegister {
    pub const fn physical<R: [const] AsId<Register>>(reg: R) -> Self {
        XvaRegister::Physical(Register::new(reg))
    }

    pub fn size(&self, mach: &dyn Machine, mode: MachineMode) -> u64 {
        match self {
            Self::Physical(r) => mach.registers().register_size(*r, mode) as u64,
            Self::Virtual(v) => v.ty.size,
        }
    }

    pub fn align(&self, mach: &dyn Machine, mode: MachineMode) -> u64 {
        match self {
            Self::Physical(r) => mach.registers().register_align(*r, mode) as u64,
            Self::Virtual(v) => v.ty.align,
        }
    }

    pub fn category(&self, mach: &dyn Machine, mode: MachineMode) -> XvaCategory {
        match self {
            Self::Physical(r) => mach.registers().register_category(*r, mode),
            Self::Virtual(v) => v.ty.category,
        }
    }

    pub fn ty(&self, mach: &dyn Machine, mode: MachineMode) -> XvaType {
        match self {
            Self::Physical(r) => XvaType {
                size: mach.registers().register_size(*r, mode) as u64,
                align: mach.registers().register_align(*r, mode) as u64,
                category: mach.registers().register_category(*r, mode),
            },
            Self::Virtual(v) => v.ty,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaDest {
    pub id: u32,
    pub ty: XvaType,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaFunction {
    pub params: Vec<XvaRegister>,
    pub preserve_regs: Vec<XvaRegister>,
    pub return_reg: Vec<XvaRegister>,
    pub statement: Vec<XvaStatement>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaFile {
    pub weak_decls: Vec<Symbol>,
    pub functions: Vec<XvaFunctionDef>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Linkage {
    External,
    Internal,
    Weak,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaFunctionDef {
    pub body: XvaFunction,
    pub linkage: Linkage,
    pub label: Symbol,
    pub section: XvaSection,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaSection {
    Global,
    Explicit(Symbol),
    PerFunction,
}
