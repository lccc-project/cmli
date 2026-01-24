use std::num::NonZeroU64;

use crate::{
    mach::{Machine, MachineMode, MachineSpec, RegisterSpec},
    target::{TargetInfo, TargetProperties},
    traits::{AsId, IdType, Name},
};

pub trait CompilerSpec: MachineSpec {
    type Machine: MachineSpec<
            Opcode = Self::Opcode,
            Register = Self::Register,
            MachineMode = Self::MachineMode,
        >;
}

pub struct CompilerContext {
    pub mode: MachineMode,
    pub properties: TargetInfo,
    pub property_overrides: TargetProperties,
}

pub trait Compiler {
    fn machine(&self) -> &dyn Machine;
}

impl<C: CompilerSpec> Compiler for C {
    fn machine(&self) -> &dyn Machine {
        self
    }
}
