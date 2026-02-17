use crate::{
    fmt::pretty_print_list,
    instr::{Address, Instruction, RegisterKind},
    intern::Symbol,
    mach::{Machine, MachineMode, Register},
    traits::{AsId, IdType as _},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaTrap {
    Unreachable,
    Abort,
}

impl core::fmt::Display for XvaTrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreachable => f.write_str("unreachable"),
            Self::Abort => f.write_str("abort"),
        }
    }
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

impl core::fmt::Display for BarrierKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";
        for flag in self.iter() {
            f.write_str(sep)?;
            sep = ", ";
            bitflags::bitflags_match! (flag,
                {
                    BarrierKind::PROPAGATE_THROUGH => f.write_str("propagate through"),
                    BarrierKind::ELIDE_INSTRS => f.write_str("intrusctions"),
                    BarrierKind::ELIDE_REGISTERS => f.write_str("registers"),
                    BarrierKind::ELIDE_STORE => f.write_str("stores"),
                    BarrierKind::DO_NOT_OPTIMIZE => f.write_str("optimize"),
                    _ => Ok(())
                }
            )?;
        }

        Ok(())
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

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaStatement> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            XvaStatement::Label(symbol) => f.write_fmt(format_args!("{symbol}:")),
            XvaStatement::Expr(expr) => PrettyPrinter(expr, self.1).fmt(f),
            XvaStatement::Write(op, reg) => f.write_fmt(format_args!(
                "write {}, {}",
                PrettyPrinter(op, self.1),
                PrettyPrinter(reg, self.1)
            )),
            XvaStatement::Jump(symbol) => f.write_fmt(format_args!("jump {symbol}")),
            XvaStatement::Tailcall { dest, params } => f.write_fmt(format_args!(
                "tailcall {} ({})",
                PrettyPrinter(dest, self.1),
                pretty_print_list(params, ", ", self.1),
            )),
            XvaStatement::Call {
                dest,
                params,
                ret_val,
                call_clobber_regs,
            } => f.write_fmt(format_args!(
                "call {} ({}) -> {} clobbers [{}]",
                PrettyPrinter(dest, self.1),
                pretty_print_list(params, ", ", self.1),
                pretty_print_list(ret_val, ", ", self.1),
                pretty_print_list(call_clobber_regs, ", ", self.1)
            )),
            XvaStatement::Return => f.write_str("return"),
            XvaStatement::Trap(trap) => f.write_fmt(format_args!("trap {trap}")),
            XvaStatement::RawInstr(instruction) => todo!(),
            XvaStatement::OptGate(barrier_kind, id) => {
                f.write_fmt(format_args!("opt barrier {id} {{{barrier_kind}}}"))
            }
            XvaStatement::EndOptGate(id) => f.write_fmt(format_args!("end opt barrier {id}")),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaExpr {
    pub dest: XvaRegister,
    pub dest2: Option<XvaRegister>,
    pub op: XvaOpcode,
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaExpr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        PrettyPrinter(&self.0.dest, self.1).fmt(f)?;

        if let Some(dest2) = self.0.dest2 {
            f.write_str(", ")?;
            PrettyPrinter(&dest2, self.1).fmt(f)?;
        }

        f.write_str(" = ")?;

        PrettyPrinter(&self.0.op, self.1).fmt(f)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaConst {
    Bits(u64),
    Label(Symbol),
    Global(Symbol, i64),
}

impl core::fmt::Display for XvaConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XvaConst::Bits(v) => f.write_fmt(format_args!("bits {v}")),
            XvaConst::Label(symbol) => f.write_fmt(format_args!("label {symbol}")),
            XvaConst::Global(symbol, val) => f.write_fmt(format_args!("global {symbol}+{val}")),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaOperand {
    Register(XvaRegister),
    Const(XvaConst),
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaOperand> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            XvaOperand::Register(reg) => PrettyPrinter(reg, self.1).fmt(f),
            XvaOperand::Const(cn) => cn.fmt(f),
        }
    }
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

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaOpcode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            XvaOpcode::ZeroInit => f.write_str("zeroinit"),
            XvaOpcode::Const(xva) => f.write_fmt(format_args!("const {xva}")),
            XvaOpcode::Uninit => f.write_str("uninit"),
            XvaOpcode::Alloca(ty) => f.write_fmt(format_args!("alloca {ty}")),
            XvaOpcode::Move(reg) => {
                f.write_fmt(format_args!("move {}", PrettyPrinter(reg, self.1)))
            }
            XvaOpcode::ComputeAddr { base, size, index } => f.write_fmt(format_args!(
                "compaddr {} + {size}*{}",
                PrettyPrinter(base, self.1),
                PrettyPrinter(index, self.1)
            )),
            XvaOpcode::BinaryOp { op, left, right } => f.write_fmt(format_args!(
                "{op} {}, {}",
                PrettyPrinter(left, self.1),
                PrettyPrinter(right, self.1)
            )),
            XvaOpcode::CheckedBinaryOp {
                op,
                mode,
                left,
                right,
            } => f.write_fmt(format_args!(
                "checked {op} {mode} {}, {}",
                PrettyPrinter(left, self.1),
                PrettyPrinter(right, self.1)
            )),
            XvaOpcode::UnaryOp { op, left } => {
                f.write_fmt(format_args!("{op} {}", PrettyPrinter(left, self.1)))
            }
            XvaOpcode::Read(op) => f.write_fmt(format_args!("read {}", PrettyPrinter(op, self.1))),
            XvaOpcode::UMul { left, right } => f.write_fmt(format_args!(
                "umul {}, {}",
                PrettyPrinter(left, self.1),
                PrettyPrinter(right, self.1)
            )),
            XvaOpcode::SMul { left, right } => f.write_fmt(format_args!(
                "smul {}, {}",
                PrettyPrinter(left, self.1),
                PrettyPrinter(right, self.1)
            )),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl core::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => f.write_str("neg"),
            Self::Not => f.write_str("not"),
        }
    }
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

impl core::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => f.write_str("add"),
            Self::Sub => f.write_str("sub"),
            Self::And => f.write_str("and"),
            Self::Or => f.write_str("or"),
            Self::Xor => f.write_str("xor"),
            Self::ShiftLeft(behaviour) => {
                f.write_str("shl ")?;
                behaviour.fmt(f)
            }
            Self::ShiftRight(behaviour) => {
                f.write_str("shr ")?;
                behaviour.fmt(f)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CheckMode {
    CheckSignedOverflow,
    CheckUnsignedOverflow,
}

impl core::fmt::Display for CheckMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CheckSignedOverflow => f.write_str("sv"),
            Self::CheckUnsignedOverflow => f.write_str("uv"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ShiftBehaviour {
    AssumeQuantity,
    WrapQuantity,
    UnboundQuantity,
}

impl core::fmt::Display for ShiftBehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssumeQuantity => f.write_str("unchecked"),
            Self::WrapQuantity => f.write_str("wrap"),
            Self::UnboundQuantity => f.write_str("unbound"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaType {
    pub size: u64,
    pub align: u64,
    pub category: XvaCategory,
}

impl core::fmt::Display for XvaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.category.fmt(f)?;

        f.write_str("{size: ")?;
        self.size.fmt(f)?;
        f.write_str(", align: ")?;
        self.align.fmt(f)?;
        f.write_str("}")
    }
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

impl core::fmt::Display for XvaCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XvaCategory::Null => f.write_str("null"),
            XvaCategory::Condition => f.write_str("condition"),
            XvaCategory::Int => f.write_str("int"),
            XvaCategory::Float => f.write_str("float"),
            XvaCategory::VectorAny => f.write_str("vector"),
            XvaCategory::VectorInt => f.write_str("vector<int>"),
            XvaCategory::VectorFloat => f.write_str("vector<float>"),
            XvaCategory::Aggregate => f.write_str("aggregate"),
            XvaCategory::Custom(_) => f.write_str("custom"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaRegister {
    Physical(Register),
    Virtual(XvaDest),
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaRegister> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            XvaRegister::Physical(reg) => f.write_str(self.1.registers().name_of(*reg)),
            XvaRegister::Virtual(virt) => virt.fmt(f),
        }
    }
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

impl core::fmt::Display for XvaDest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("#")?;
        self.id.fmt(f)?;
        f.write_str(": ")?;
        self.ty.fmt(f)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaFunction {
    pub params: Vec<XvaRegister>,
    pub preserve_regs: Vec<XvaRegister>,
    pub return_reg: Vec<XvaRegister>,
    pub statement: Vec<XvaStatement>,
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaFunction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PARAMS: ")?;

        let mut sep = "";

        for reg in &self.0.params {
            f.write_str(sep)?;
            sep = " ";
            PrettyPrinter(reg, self.1).fmt(f)?;
        }
        f.write_str("\n")?;

        f.write_str("PRESERVE REGISTERS: ")?;

        sep = "";

        for reg in &self.0.preserve_regs {
            f.write_str(sep)?;
            sep = " ";
            PrettyPrinter(reg, self.1).fmt(f)?;
        }
        f.write_str("\n")?;

        f.write_str("RETURN: ")?;

        sep = "";

        for reg in &self.0.return_reg {
            f.write_str(sep)?;
            sep = " ";
            PrettyPrinter(reg, self.1).fmt(f)?;
        }
        f.write_str("\n")?;

        for stmt in &self.0.statement {
            f.write_str("\t")?;
            PrettyPrinter(stmt, self.1).fmt(f)?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct XvaFile {
    pub weak_decls: Vec<Symbol>,
    pub functions: Vec<XvaFunctionDef>,
    pub objects: Vec<XvaObjectDef>,
}

impl XvaFile {
    pub fn pretty_print<'a>(&'a self, mach: &'a dyn Machine) -> PrettyPrinter<'a, XvaFile> {
        PrettyPrinter(self, mach)
    }
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaFile> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for sym in &self.0.weak_decls {
            f.write_fmt(format_args!("extern weak {sym};\n"))?;
        }

        for func in &self.0.functions {
            PrettyPrinter(func, self.1).fmt(f)?;
        }

        for def in &self.0.objects {
            PrettyPrinter(def, self.1).fmt(f)?;
        }
        Ok(())
    }
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

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaFunctionDef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let XvaFunctionDef {
            body,
            linkage,
            label,
            section,
        } = self.0;
        f.write_fmt(format_args!(
            "{linkage} function {label} (section {section}):\n"
        ))?;

        PrettyPrinter(body, self.1).fmt(f)?;

        f.write_fmt(format_args!("end function {label}\n"))
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaSection {
    Text,
    RoData,
    Data,
    Explicit(Symbol),
    PrivateText,
    Common,
    TlsData,
}

impl core::fmt::Display for Linkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Linkage::External => f.write_str("external"),
            Linkage::Internal => f.write_str("internal"),
            Linkage::Weak => f.write_str("weak"),
        }
    }
}

impl core::fmt::Display for XvaSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XvaSection::Text => f.write_str("text"),
            XvaSection::RoData => f.write_str("rodata"),
            XvaSection::Data => f.write_str("data"),
            XvaSection::Explicit(name) => f.write_str(name.as_str()),
            XvaSection::PrivateText => f.write_str("private"),
            XvaSection::Common => f.write_str("common"),
            XvaSection::TlsData => f.write_str("tls data"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaObjectDef {
    pub ty: XvaType,
    pub body: Vec<u8>,
    pub relocs: Vec<XvaRelocation>,
    pub linkage: Linkage,
    pub label: Symbol,
    pub section: XvaSection,
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, XvaObjectDef> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.linkage.fmt(f)?;
        f.write_str(" def ")?;
        f.write_str(&self.0.label)?;
        f.write_str(" as ")?;
        self.0.ty.fmt(f)?;
        f.write_str(" (in ")?;
        self.0.section.fmt(f)?;
        f.write_str(")")?;

        for (i, b) in self.0.body.iter().enumerate() {
            if (i & 15) == 0 {
                f.write_str("\n\t")?;
            } else {
                f.write_str(" ")?;
            }
            f.write_fmt(format_args!("{b:02x}"))?;
        }

        f.write_str("\nend def ")?;
        f.write_str(&self.0.label)?;
        f.write_str("\n")
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct XvaRelocation {
    pub offset: usize,
    pub addr: Address,
}

use crate::fmt::PrettyPrinter;
