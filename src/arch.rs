use crate::{
    helpers::*,
    instr::Opcodes,
    traits::{IdentityName, Unique},
};

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

    fn vector_support(&self) -> Option<&(dyn VectorSupport + '_)> {
        None
    }
}

impl<A: Arch + ?Sized> IdentityName for A {
    fn name(&self) -> &str {
        <A as Arch>::name(self)
    }
}

impl_singleton_hash_eq!(Arch);
impl_identity_debug_display!(Arch);

pub trait VectorSupport: Arch {
    fn max_vector_size(&self) -> u16;
}

pub mod x86;
