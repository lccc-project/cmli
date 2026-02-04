use std::num::NonZeroU64;

use crate::{
    mach::{Machine, MachineMode, MachineSpec, RegisterSpec},
    target::{PropertyValue, TargetInfo, TargetProperties},
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
}

impl<C: CompilerSpec> Compiler for C {
    fn machine(&self) -> &dyn Machine {
        self
    }
}
