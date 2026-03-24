use std::{any::Any, collections::HashSet};

use crate::xva::{BarrierKind, XvaBasicBlock, XvaFile, XvaFunction, XvaRegister, XvaStatement};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum XvaOptPhase {
    AfterLower,
    BeforeRegalloc,
    AfterRegalloc,
    BeforeLegalize,
    AfterLegalize,
    BeforeMce,
    Mce,
}

pub trait State: Any {
    fn reset_registers(&mut self);
    fn mark_has_value(&mut self, reg: XvaRegister);
    fn push_gate(&mut self, ty: BarrierKind, num: u32);
    fn pop_gate(&mut self, num: u32);
}

pub trait XvaOpt {
    fn phases(&self) -> &[XvaOptPhase];
    fn cost(&self) -> usize;

    fn make_state(&self) -> Box<dyn State>;
}

pub trait XvaFunctionOpt: XvaOpt {
    fn optimize_function(&self, state: &mut dyn State, func: &mut XvaFunction, phase: XvaOptPhase);
}

pub trait XvaBasicBlockOpt: XvaOpt {
    fn optimize_basic_block(
        &self,
        state: &mut dyn State,
        bb: &mut XvaBasicBlock,
        phase: XvaOptPhase,
    );
}

pub trait XvaStatementOpt: XvaOpt {
    fn optimize_statement(
        &self,
        state: &mut dyn State,
        stmt: &mut XvaStatement,
        phase: XvaOptPhase,
    );
}

impl<X: XvaBasicBlockOpt> XvaFunctionOpt for X {
    fn optimize_function(&self, state: &mut dyn State, func: &mut XvaFunction, phase: XvaOptPhase) {
        for block in &mut func.body {
            state.reset_registers();
            for &live in &block.live_at_start {
                state.mark_has_value(live);
            }
            self.optimize_basic_block(state, block, phase);
        }
    }
}

impl<X: XvaStatementOpt> XvaBasicBlockOpt for X {
    fn optimize_basic_block(
        &self,
        state: &mut dyn State,
        func: &mut XvaBasicBlock,
        phase: XvaOptPhase,
    ) {
        match &mut func.body {
            super::XvaBlockBody::Statement(stmts) => {
                for stmt in stmts {
                    self.optimize_statement(state, stmt, phase);
                }
            }
        }
    }
}

pub mod pass;

pub const ALL_PASSES: &[&dyn XvaFunctionOpt] = &[&pass::FoldRegisterPass, &pass::RemoveUnused];

fn flatten_statements(dest: &mut Vec<XvaStatement>, stmts: Vec<XvaStatement>) {
    for stmt in stmts {
        match stmt {
            XvaStatement::Elaborated(stmts) => flatten_statements(dest, stmts),
            stmt => dest.push(stmt),
        }
    }
}

pub fn run_passes<'a>(
    passes: impl Iterator<Item = &'a (dyn XvaFunctionOpt + 'a)>,
    phase: XvaOptPhase,
    prg: &mut XvaFile,
    mut fuel: usize,
) {
    let mut modified_funcs = HashSet::new();
    for pass in passes {
        if !pass.phases().contains(&phase) {}
        let cost = pass.cost();

        if cost > fuel {
            continue;
        }
        fuel -= cost;
        for func in &mut prg.functions {
            modified_funcs.insert(func as *mut _);
            let mut state = pass.make_state();
            pass.optimize_function(&mut *state, &mut func.body, phase);
        }
    }

    if !modified_funcs.is_empty() {
        for func in &mut prg.functions {
            if modified_funcs.contains(&(func as *mut _)) {
                for block in &mut func.body.body {
                    match &mut block.body {
                        super::XvaBlockBody::Statement(xva_statements) => {
                            let stmts = core::mem::take(xva_statements);
                            flatten_statements(xva_statements, stmts);
                        }
                    }
                }
            }
        }
    }
}
