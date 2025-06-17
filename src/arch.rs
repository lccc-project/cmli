use crate::{
    helpers::*,
    instr::Opcodes,
    traits::{IdentityName, Unique},
};

def_id_type!(MachId);
def_id_type!(FeatureId);

#[derive(MachId, Copy, Clone, Hash, PartialEq, Eq)]
pub enum UniformMachine {
    Simple,
}

pub trait Arch: IdentityName + Unique {
    fn name(&self) -> &str;
    fn address_width(&self) -> u16;
    fn data_address_width(&self) -> u16 {
        self.address_width()
    }
    fn code_address_width(&self) -> u16 {
        self.address_width()
    }
    fn natural_int_width(&self) -> u16 {
        self.address_width()
    }
    fn opcodes(&self) -> Option<&dyn Opcodes>;

    fn float_support(&self) -> Option<&(dyn FloatSupport + '_)> {
        None
    }

    fn vector_support(&self) -> Option<&(dyn VectorSupport + '_)> {
        None
    }

    fn machines(&self) -> Option<&(dyn MachineInfo + '_)>;
}

impl<A: Arch + ?Sized> IdentityName for A {
    fn name(&self) -> &str {
        <A as Arch>::name(self)
    }
}

impl_singleton_hash_eq!(Arch);
impl_identity_debug_display!(Arch);

pub trait FloatSupport: Arch {}

pub trait VectorSupport: Arch {
    fn max_vector_size(&self) -> u16;
}

pub trait MachineInfo: Arch {
    fn base_machine(&self) -> MachId;
    fn all_machines(&self) -> &[MachId];
    fn all_features(&self) -> &[FeatureId];
    fn machine_features(&self, mach: MachId) -> &[FeatureId];
    fn mach_by_name(&self, name: &str) -> Option<MachId>;
    fn mach_name(&self, mach: MachId) -> &str;
    fn feature_by_name(&self, name: &str) -> Option<FeatureId>;
    fn feature_name(&self, feature: FeatureId) -> &str;
}

#[cfg(feature = "x86")]
pub mod x86;
