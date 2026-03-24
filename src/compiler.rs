use std::{collections::HashSet, num::NonZeroU64};

use crate::{
    mach::{Machine, MachineMode, MachineSpec, Register, RegisterSpec},
    target::{PropertyValue, TargetInfo, TargetProperties},
    traits::{AsId, IdType, Name},
    xva::XvaCategory,
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
}

pub struct CompilerContext {
    pub mode: MachineMode,
    pub properties: TargetInfo,
    pub property_overrides: TargetProperties,
    pub target_features: HashSet<String>,
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
}
