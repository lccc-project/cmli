//! Compilation support for architectures
use std::{collections::HashSet, num::NonZeroU64};

use crate::{
    instr::{Address, AddressKind, Instruction}, mach::{FeatureSet, Machine, MachineMode, MachineSpec, Register, RegisterSpec}, target::{PropertyValue, TargetInfo, TargetProperties}, traits::{AsId, IdType, Name}, xva::{NoopKind, XvaCategory, XvaFrameProperties, XvaRegister, XvaStatement}
};



pub trait CompilerSpec: MachineSpec {
    type Machine: MachineSpec<
            Opcode = Self::Opcode,
            Register = Self::Register,
            MachineMode = Self::MachineMode,
        >;

    fn available_registers(
        &self,
        context: &CompilerContext,
        mode: Self::MachineMode,
        cat: XvaCategory,
        size: u32,
    ) -> Option<&[Register]>;

    fn promote_size(
        &self,
        context: &crate::compiler::CompilerContext,
        mode: Self::MachineMode,
        cat: XvaCategory,
        size: u32,
    ) -> Option<u32>;

    fn lower_mce(&self, stmt: &mut XvaStatement, mode: Self::MachineMode, context: &CompilerContext, features: &FeatureSet);

    fn lower_epilogue(&self, frame: &XvaFrameProperties, mode: Self::MachineMode) -> Vec<XvaStatement>;
    fn emit_prologue(&self, frame: &mut XvaFrameProperties, mode: Self::MachineMode) -> Vec<Instruction>;

    /// Helper function for implementing [`Self::lower_mce`]
    /// 
    /// ## Panics
    /// Panics if a virtual register is passed, or if it is of an unexpected type
    fn areg(reg: XvaRegister) -> Self::Register {
        let XvaRegister::Physical(reg) = reg else {
            panic!("Encountered virtual register")
        };

        reg.downcast().expect("Bad register kind")
    }
}

pub struct CompilerContext {
    pub mode: MachineMode,
    pub properties: TargetInfo,
    pub property_overrides: TargetProperties,
    pub target_features: HashSet<String>,
    pub global_address_kind: AddressKind,
    pub global_call_address_kind: AddressKind,
    pub local_address_kind: AddressKind,
    pub global_tls_kind: AddressKind,
    pub local_tls_kind: AddressKind,
}

impl CompilerContext {
    pub fn property<S: AsRef<str> + ?Sized>(&self, key: &S) -> Option<&PropertyValue> {
        let st = key.as_ref();
        if let Some(prop) = self.property_overrides.global_properties.get(st) {
            Some(prop)
        } else {
            self.properties.properties.global_properties.get(st)
        }
    }
}

pub trait Compiler {
    fn machine(&self) -> &dyn Machine;

    fn available_registers(
        &self,
        context: &CompilerContext,
        cat: XvaCategory,
        size: u32,
    ) -> Option<&[Register]>;

    fn promote_size(
        &self,
        context: &crate::compiler::CompilerContext,
        cat: XvaCategory,
        size: u32,
    ) -> Option<u32>;

    fn mce_lower(&self, xva: &mut XvaStatement, frame: &XvaFrameProperties, mode: &CompilerContext);

    fn emit_prologue(&self, frame: &mut XvaFrameProperties, mode: MachineMode) -> Vec<Instruction>;
}

impl<C: CompilerSpec> Compiler for C {
    fn machine(&self) -> &dyn Machine {
        self
    }

    fn available_registers(
        &self,
        context: &CompilerContext,
        cat: XvaCategory,
        size: u32,
    ) -> Option<&[Register]> {
        CompilerSpec::available_registers(
            self,
            context,
            context
                .mode
                .downcast::<<C as MachineSpec>::MachineMode>()
                .unwrap(),
            cat,
            size,
        )
    }

    fn promote_size(
        &self,
        context: &crate::compiler::CompilerContext,
        cat: XvaCategory,
        size: u32,
    ) -> Option<u32> {
        <C as CompilerSpec>::promote_size(
            self,
            context,
            context
                .mode
                .downcast::<<C as MachineSpec>::MachineMode>()
                .unwrap(),
            cat,
            size,
        )
    }

    fn mce_lower(&self, xva: &mut XvaStatement, frame: &XvaFrameProperties, context: &CompilerContext) {
        let mode = context.mode;
        let mmode = context.mode.downcast::<<C as MachineSpec>::MachineMode>().unwrap();
        match xva {
            XvaStatement::Elaborated(stmts) => {
                for stmt in stmts {
                    self.mce_lower(stmt, frame, context);
                }
            },
            XvaStatement::Noop(NoopKind::Normal) => {}

            XvaStatement::Write(_, ty, _) => {
                if ty.size > 0 {
                    self.lower_mce(xva, mmode, context, &frame.features);
                }
            }
            
            XvaStatement::Expr(expr) => {
                if expr.dest.size(self.machine(), mode) > 0 {
                    self.lower_mce(xva, mmode, context, &frame.features);
                }
            }
            XvaStatement::RawInstr(_) |
            XvaStatement::OptGate(_, _) |
             XvaStatement::Use(_, _) |
            XvaStatement::EndOptGate(_) => {},
            XvaStatement::Fallthrough(_) => {
                *xva = XvaStatement::Elaborated(vec![])
            },

            XvaStatement::Return | XvaStatement::Tailcall { .. } => {
                let mut stmts = if frame.has_prologue { 
                    self.lower_epilogue(frame, mmode) 
                } else {
                    Vec::new()
                };

                self.lower_mce(xva, mmode, context, &frame.features);

                stmts.push(core::mem::take(xva));

                *xva = XvaStatement::Elaborated(stmts);
            }

            

            stmt => {
                self.lower_mce(stmt, mmode, context, &frame.features);
            }
        }
    }

    fn emit_prologue(&self, frame: &mut XvaFrameProperties, mode: MachineMode) -> Vec<Instruction> {
        let mmode = mode.downcast::<<C as MachineSpec>::MachineMode>().unwrap();
        <Self as CompilerSpec>::emit_prologue(self, frame, mmode)
    }
}
