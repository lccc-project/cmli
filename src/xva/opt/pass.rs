use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

use crate::{
    mach::{Machine, MachineMode, RegisterSpec},
    xva::{
        self, BarrierKind, UseKind, XvaConst, XvaExpr, XvaOpcode, XvaOperand, XvaRegister,
        XvaStatement,
        opt::{State, XvaFunctionOpt, XvaOpt, XvaOptPhase, XvaStatementOpt},
    },
};

#[derive(Default)]
pub struct NoState;

impl State for NoState {
    fn reset_registers(&mut self) {}

    fn mark_has_value(&mut self, reg: XvaRegister) {}

    fn push_gate(&mut self, ty: BarrierKind, num: u32) {}

    fn pop_gate(&mut self, num: u32) {}
}

pub struct PassState {
    pub live_register_values: HashMap<XvaRegister, LiveValue>,
    pub opt_gate_state: Vec<(u32, BarrierKind)>,
    pub mode: MachineMode,
}

impl PassState {
    pub fn new(mode: MachineMode) -> Self {
        Self{live_register_values: HashMap::new(), opt_gate_state: Vec::new(), mode}
    }
}

impl PassState {
    /// Tests the current opt-gate state. Returns false if any bit set in `forbids` is enforced by a live opt gate
    pub fn test_barrier(&self, forbids: BarrierKind) -> bool {
        let state = self
            .opt_gate_state
            .iter()
            .map(|&(_, kind)| kind)
            .fold(BarrierKind::empty(), BarrierKind::union);

        return !state.intersects(forbids);
    }

    pub fn test_uninit(&self, reg: XvaRegister) -> bool {
        self.live_register_values
            .get(&reg)
            .copied()
            .unwrap_or(LiveValue::Uninit)
            == LiveValue::Uninit
    }
}

impl State for PassState {
    fn mark_has_value(&mut self, reg: XvaRegister) {
        self.live_register_values.insert(reg, LiveValue::Unknown);
    }

    fn reset_registers(&mut self) {
        self.live_register_values.clear();
    }

    fn push_gate(&mut self, ty: BarrierKind, num: u32) {
        self.opt_gate_state.push((num, ty));
    }

    fn pop_gate(&mut self, num: u32) {
        let (num_stored, _) = self.opt_gate_state.pop().unwrap();

        assert_eq!(num_stored, num);
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LiveValue {
    Unknown,
    Uninit,
    ZeroInit,
    Const(XvaConst),
    CopyReg(XvaRegister),
}

impl LiveValue {
    pub fn replace_move_opcode(&self) -> Option<XvaOpcode> {
        match self {
            LiveValue::Unknown => None,
            LiveValue::Uninit => Some(XvaOpcode::Uninit),
            LiveValue::ZeroInit => Some(XvaOpcode::ZeroInit),
            LiveValue::Const(xva_const) => Some(XvaOpcode::Const(*xva_const)),
            LiveValue::CopyReg(xva_register) => Some(XvaOpcode::Move(*xva_register)),
        }
    }

    pub fn into_operand(&self, reg: XvaRegister) -> XvaOperand {
        match self {
            LiveValue::Unknown => XvaOperand::Register(reg),
            LiveValue::Uninit | LiveValue::ZeroInit => XvaOperand::Const(XvaConst::Bits(0)),
            LiveValue::Const(xva_const) => XvaOperand::Const(*xva_const),
            LiveValue::CopyReg(reg) => XvaOperand::Register(*reg),
        }
    }
}

pub struct FoldRegisterPass;

impl XvaOpt for FoldRegisterPass {
    fn cost(&self) -> usize {
        10
    }
    fn phases(&self) -> &[super::XvaOptPhase] {
        &[super::XvaOptPhase::AfterLower, XvaOptPhase::BeforeRegalloc]
    }

    fn make_state(&self, mode: MachineMode) -> Box<dyn State> {
        Box::new(PassState::new(mode))
    }
}

impl XvaStatementOpt for FoldRegisterPass {
    fn optimize_statement(
        &self,
        state: &mut dyn State,
        stmt: &mut crate::xva::XvaStatement,
        phase: XvaOptPhase,
        mach: &dyn Machine,
    ) {
        let state = (state as &mut dyn Any).downcast_mut::<PassState>().unwrap();
        match stmt {
            crate::xva::XvaStatement::Expr(xva_expr) => {
                if !state.test_barrier(BarrierKind::PROPAGATE_THROUGH) {
                    state.mark_has_value(xva_expr.dest);
                    if let Some(dest2) = xva_expr.dest2 {
                        state.mark_has_value(dest2);
                    }
                    return;
                }
                match &mut xva_expr.op {
                    crate::xva::XvaOpcode::ZeroInit => {
                        state
                            .live_register_values
                            .insert(xva_expr.dest, LiveValue::ZeroInit);

                        if let Some(dest2) = xva_expr.dest2 {
                            state
                                .live_register_values
                                .insert(dest2, LiveValue::ZeroInit);
                        }
                    }
                    crate::xva::XvaOpcode::Const(xva_const) => {
                        state
                            .live_register_values
                            .insert(xva_expr.dest, LiveValue::Const(*xva_const));
                    }
                    crate::xva::XvaOpcode::Uninit => {
                        state
                            .live_register_values
                            .insert(xva_expr.dest, LiveValue::Uninit);
                        if let Some(dest2) = xva_expr.dest2 {
                            state.live_register_values.insert(dest2, LiveValue::Uninit);
                        }
                    }
                    crate::xva::XvaOpcode::Move(xva_register) => {
                        match state
                            .live_register_values
                            .get(xva_register)
                            .copied()
                            .unwrap_or(LiveValue::Uninit)
                        {
                            LiveValue::Unknown => {
                                state
                                    .live_register_values
                                    .insert(xva_expr.dest, LiveValue::CopyReg(*xva_register));
                            }
                            val => {
                                state.live_register_values.insert(xva_expr.dest, val);

                                if state.test_barrier(BarrierKind::ELIDE_REGISTERS)
                                    && let Some(op) = val.replace_move_opcode()
                                {
                                    xva_expr.op = op;
                                }
                            }
                        }
                    }
                    crate::xva::XvaOpcode::BinaryOp { left, right, .. }
                    | crate::xva::XvaOpcode::CheckedBinaryOp { left, right, .. } => {
                        let is_uninit = if let XvaOperand::Register(reg) = right {
                            state.test_uninit(*reg) || state.test_uninit(*left)
                        } else {
                            state.test_uninit(*left)
                        };
                        if is_uninit {
                            state
                                .live_register_values
                                .insert(xva_expr.dest, LiveValue::Uninit);
                            xva_expr.op = XvaOpcode::Uninit;
                            return;
                        }

                        if let LiveValue::CopyReg(r) = state
                            .live_register_values
                            .get(left)
                            .copied()
                            .unwrap_or(LiveValue::Uninit)
                        {
                            *left = r;
                        }

                        match right {
                            XvaOperand::Register(reg) => {
                                let reg = *reg;
                                let val = state
                                    .live_register_values
                                    .get(&reg)
                                    .copied()
                                    .unwrap_or(LiveValue::Uninit);

                                *right = val.into_operand(reg);
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        state.mark_has_value(xva_expr.dest);
                        if let Some(dest2) = xva_expr.dest2 {
                            state.mark_has_value(dest2);
                        }
                    }
                }
            }
            crate::xva::XvaStatement::Write(_, xva_register) => {
                if !state
                    .test_barrier(BarrierKind::PROPAGATE_THROUGH | BarrierKind::ELIDE_REGISTERS)
                {
                    if let LiveValue::CopyReg(reg) = state
                        .live_register_values
                        .get(xva_register)
                        .copied()
                        .unwrap_or(LiveValue::Uninit)
                    {
                        *xva_register = reg;
                    }
                }
            }
            crate::xva::XvaStatement::Jump(_) => {}
            crate::xva::XvaStatement::Tailcall { .. } => {}
            crate::xva::XvaStatement::Call {
                dest,
                params,
                ret_val,
                call_clobber_regs,
            } => {
                for reg in call_clobber_regs.into_registers(mach, state.mode) {
                    if state.test_barrier(BarrierKind::ELIDE_STORE) {
                        state.live_register_values.remove(&XvaRegister::Physical(reg));
                    } else {
                        state.mark_has_value(XvaRegister::Physical(reg)); // Clobbered Registers become unknown not uninit if we're in DNO
                    }
                }
                for reg in ret_val.into_registers(mach, state.mode) {
                    state.mark_has_value(XvaRegister::Physical(reg));
                }
            }
            crate::xva::XvaStatement::Return => {}
            crate::xva::XvaStatement::Trap(_) => {
                if state.test_barrier(BarrierKind::ELIDE_STORE) {
                    state.reset_registers();
                }
            }
            crate::xva::XvaStatement::RawInstr(instruction) => {
                for opr in instruction.operands() {
                    match opr {
                        crate::instr::Operand::Register(register) => {
                            state.mark_has_value(XvaRegister::Physical(*register));
                        }
                        _ => {}
                    }
                }
            }
            crate::xva::XvaStatement::OptGate(barrier_kind, num) => {
                state.push_gate(*barrier_kind, *num);
            }
            crate::xva::XvaStatement::EndOptGate(num) => {
                state.pop_gate(*num);
            }
            crate::xva::XvaStatement::Noop(_) => {}
            crate::xva::XvaStatement::Elaborated(xva_statements) => {
                for stmt in xva_statements {
                    self.optimize_statement(state, stmt, phase, mach);
                }
            }
            crate::xva::XvaStatement::Use(regs, kind) => match kind {
                UseKind::Write | UseKind::ReadWrite
                    if state.test_barrier(BarrierKind::PROPAGATE_THROUGH) =>
                {
                    for reg in regs {
                        state.mark_has_value(*reg);
                    }
                }
                _ => {}
            },
            XvaStatement::Fallthrough(_) => {}
        }
    }
}
pub struct RemoveUnusedState {
    pass: PassState,
    used_regs: HashSet<XvaRegister>,
    return_regs: Vec<XvaRegister>,
}

impl RemoveUnusedState {
    pub fn new(mode: MachineMode) -> Self {
        Self{pass: PassState::new(mode), used_regs: HashSet::new(), return_regs: Vec::new()}
    }
}

impl State for RemoveUnusedState {
    fn reset_registers(&mut self) {
        self.pass.reset_registers();
    }

    fn mark_has_value(&mut self, reg: XvaRegister) {
        self.pass.mark_has_value(reg);
    }

    fn push_gate(&mut self, ty: BarrierKind, num: u32) {
        self.pass.push_gate(ty, num);
    }

    fn pop_gate(&mut self, num: u32) {
        self.pass.pop_gate(num);
    }
}

pub struct RemoveUnused;

impl XvaOpt for RemoveUnused {
    fn cost(&self) -> usize {
        15
    }

    fn make_state(&self, mode: MachineMode) -> Box<dyn State> {
        Box::new(RemoveUnusedState::new(mode))
    }

    fn phases(&self) -> &[XvaOptPhase] {
        &[super::XvaOptPhase::AfterLower, XvaOptPhase::BeforeRegalloc]
    }
}

impl RemoveUnused {
    fn collect_operand(&self, state: &mut RemoveUnusedState, opr: XvaOperand) {
        match opr {
            XvaOperand::Register(reg) => {
                state.used_regs.insert(reg);
            }
            XvaOperand::Const(_) | XvaOperand::FrameAddr(_) => {}
        }
    }

    fn collect_expr(&self, state: &mut RemoveUnusedState, expr: &XvaExpr) {
        match &expr.op {
            XvaOpcode::ZeroInit
            | XvaOpcode::Const(_)
            | XvaOpcode::Uninit
            | XvaOpcode::GetFrameAddr(_) => {}
            XvaOpcode::Move(xva_register) => {
                state.used_regs.insert(*xva_register);
            }
            XvaOpcode::Read(opr) => {
                self.collect_operand(state, *opr);
            }
            XvaOpcode::ComputeAddr { base, index, .. } => {
                self.collect_operand(state, *base);
                self.collect_operand(state, *index);
            }
            XvaOpcode::CheckedBinaryOp { left, right, .. }
            | XvaOpcode::BinaryOp { left, right, .. } => {
                state.used_regs.insert(*left);
                self.collect_operand(state, *right);
            }

            XvaOpcode::UnaryOp { left, .. } => {
                state.used_regs.insert(*left);
            }

            XvaOpcode::UMul { left, right } | XvaOpcode::SMul { left, right } => {
                state.used_regs.insert(*left);
                state.used_regs.insert(*right);
            }
        }
    }
    pub fn collect_phase(&self, state: &mut RemoveUnusedState, stmt: &XvaStatement, mach: &dyn Machine) {
        match stmt {
            xva::XvaStatement::Expr(xva_expr) => self.collect_expr(state, xva_expr),
            xva::XvaStatement::Write(opr, reg) => {
                self.collect_operand(state, *opr);
                state.used_regs.insert(*reg);
            }
            xva::XvaStatement::Jump(symbol) => {}
            xva::XvaStatement::Call { dest, params, .. }
            | xva::XvaStatement::Tailcall { dest, params } => {
                self.collect_operand(state, *dest);
                for reg in params.into_registers(mach, state.pass.mode) {
                    state.used_regs.insert(XvaRegister::Physical(reg));
                }
            }
            xva::XvaStatement::Return => {
                for reg in &state.return_regs {
                    state.used_regs.insert(*reg);
                }
            }
            xva::XvaStatement::Trap(_) => {}
            xva::XvaStatement::RawInstr(instruction) => {
                for opr in instruction.operands() {
                    match opr {
                        crate::instr::Operand::Register(register) => {
                            state.used_regs.insert(XvaRegister::Physical(*register));
                        }
                        crate::instr::Operand::AbsSymbol(_, _)
                        | crate::instr::Operand::RelSymbol(_, _)
                        | crate::instr::Operand::Immediate(_) => {}

                        crate::instr::Operand::Memory(memory_operand) => {
                            let addr = &memory_operand.addr;

                            for reg in [addr.segment, addr.base, addr.index].into_iter().flatten() {
                                state.used_regs.insert(XvaRegister::Physical(reg));
                            }
                        }
                    }
                }
            }
            xva::XvaStatement::OptGate(kind, num) => state.push_gate(*kind, *num),
            xva::XvaStatement::EndOptGate(num) => state.pop_gate(*num),
            xva::XvaStatement::Noop(_) => {}
            xva::XvaStatement::Elaborated(xva_statements) => {
                for stmt in xva_statements {
                    self.collect_phase(state, stmt, mach);
                }
            }
            xva::XvaStatement::Use(regs, use_kind) => match use_kind {
                UseKind::Read | UseKind::ReadWrite
                    if state.pass.test_barrier(BarrierKind::ELIDE_REGISTERS) =>
                {
                    for reg in regs {
                        state.used_regs.insert(*reg);
                    }
                }
                _ => {}
            },
            XvaStatement::Fallthrough(_) => {}
        }
    }

    pub fn remove_phase(&self, state: &mut RemoveUnusedState, stmt: &mut XvaStatement) {
        match stmt {
            xva::XvaStatement::OptGate(kind, num) => state.push_gate(*kind, *num),
            xva::XvaStatement::EndOptGate(num) => state.pop_gate(*num),
            XvaStatement::Expr(xva_expr) => {
                if !state.used_regs.contains(&xva_expr.dest)
                    && !xva_expr
                        .dest2
                        .map_or(false, |v| state.used_regs.contains(&v))
                {
                    *stmt = XvaStatement::Elaborated(vec![]);
                } else if let XvaOpcode::Move(reg) = &xva_expr.op {
                    if state.pass.test_barrier(BarrierKind::ELIDE_REGISTERS | BarrierKind::ELIDE_STORE | BarrierKind::ELIDE_INSTRS) && xva_expr.dest == *reg {
                        *stmt = XvaStatement::Elaborated(vec![]);
                    }
                }
            }
            _ => {}
        }
    }
}

impl XvaFunctionOpt for RemoveUnused {
    fn optimize_function(
        &self,
        state: &mut dyn State,
        func: &mut xva::XvaFunction,
        _: XvaOptPhase,
        mach: &dyn Machine
    ) {
        let state = (state as &mut dyn Any)
            .downcast_mut::<RemoveUnusedState>()
            .unwrap();

        state.return_regs = func.return_regs.into_registers(mach, state.pass.mode).map(XvaRegister::Physical).collect();

        for stmt in &mut func.body {
            match &mut stmt.body {
                xva::XvaBlockBody::Statement(xva_statements) => {
                    for stmt in xva_statements {
                        self.collect_phase(state, stmt, mach);
                    }
                }
            }
        }

        func.params.retain_all(state.used_regs.iter().filter_map(|r| {
            match r {
                XvaRegister::Physical(r) => Some(*r),
                _ => None
            }
        }), mach);

        for stmt in &mut func.body {
            stmt.live_at_start.retain(|v| state.used_regs.contains(v));
            match &mut stmt.body {
                xva::XvaBlockBody::Statement(xva_statements) => {
                    for stmt in xva_statements {
                        self.remove_phase(state, stmt);
                    }
                }
            }
        }
    }
}

pub struct OptimizeFallthrough;

impl XvaOpt for OptimizeFallthrough {
    fn phases(&self) -> &[XvaOptPhase] {
        &[XvaOptPhase::AfterLower]
    }

    fn cost(&self) -> usize {
        5
    }

    fn make_state(&self,_ :MachineMode) -> Box<dyn State> {
        Box::new(NoState)
    }
}

impl XvaFunctionOpt for OptimizeFallthrough {
    fn optimize_function(&self, _: &mut dyn State, func: &mut xva::XvaFunction, _: XvaOptPhase, _: &dyn Machine) {
        let mut labels = Vec::new();
        for bb in &func.body {
            labels.push(bb.label);
        }

        for (i, bb) in func.body.iter_mut().enumerate() {
            match &mut bb.body {
                xva::XvaBlockBody::Statement(stmts) => match stmts.last_mut() {
                    Some(XvaStatement::Jump(jmp)) => {
                        if let Some(v) = labels.get(i + 1) {
                            if v == jmp {
                                *stmts.last_mut().unwrap() = XvaStatement::Fallthrough(*jmp);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
