use crate::traits::{AsId, IdType, Name};
use std::{hash::Hasher, num::NonZeroU64};

use crate::traits::AsRawId;

#[derive(AsRawId, Copy, Clone, Debug, Hash, PartialEq, Eq, Name)]
pub enum OneMachine {
    Singleton,
}

impl const AsId<MachineMode> for OneMachine {}

const ONE_MACHINE: &[MachineMode] = as_id_array!([OneMachine::Singleton] => MachineMode);

pub trait MachineSpec: Sized {
    type Opcode: AsId<Opcode> + Name;
    const OPCODES: &[Opcode];
    type Register: AsId<Register> + Name;
    const REGISTERS: &[Register];
    type MachineMode: AsId<MachineMode> + Name;
    const MACH_MODES: &[MachineMode];

    fn name(&self) -> &'static str;
}

macro_rules! machine_helper {
    (fn $method:ident(&self) -> $ty_name:ident {
        $assoc_const:ident
    }) => {
        fn $method(&self) -> &(dyn DynList<$ty_name> + '_) {
            struct ListHelper<M>(core::marker::PhantomData<M>);
            impl<M: MachineSpec> DynList<$ty_name> for ListHelper<M> {
                fn list(&self) -> &[$ty_name] {
                    const { M::$assoc_const }
                }

                fn name_of(&self, val: $ty_name) -> &'static str {
                    match val.downcast::<M::$ty_name>() {
                        Some(val) => val.name(),
                        None => ::core::concat!("**Unknown ", ::core::stringify!($ty_anem), "**"),
                    }
                }
            }

            &ListHelper::<Self>(core::marker::PhantomData)
        }
    };
}

impl<M: MachineSpec> Machine for M {
    fn name(&self) -> &'static str {
        <Self as MachineSpec>::name(self)
    }

    machine_helper!(
        fn opcodes(&self) -> Opcode {
            OPCODES
        }
    );
    machine_helper!(
        fn registers(&self) -> Register {
            REGISTERS
        }
    );
    machine_helper!(
        fn modes(&self) -> MachineMode {
            MACH_MODES
        }
    );
}

pub trait Machine {
    fn name(&self) -> &'static str;
    fn opcodes(&self) -> &(dyn DynList<Opcode> + '_);
    fn registers(&self) -> &(dyn DynList<Register> + '_);
    fn modes(&self) -> &(dyn DynList<MachineMode> + '_);
}

macro_rules! impl_machine_helper {
    ($ty:ty) => {
        impl core::hash::Hash for $ty {
            fn hash<H: Hasher>(&self, state: &mut H) {
                core::ptr::hash(self, state);
            }
        }

        impl PartialEq for $ty {
            fn eq(&self, other: &Self) -> bool {
                core::ptr::eq(self, other)
            }
        }

        impl Eq for $ty {}

        impl core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str(self.name())
            }
        }
    };
}

impl_machine_helper!(dyn Machine + '_);
impl_machine_helper!(dyn Machine + Send + '_);
impl_machine_helper!(dyn Machine + Sync + '_);
impl_machine_helper!(dyn Machine + Send + Sync + '_);

pub trait DynList<T> {
    fn list(&self) -> &[T];

    fn name_of(&self, val: T) -> &'static str;
}

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Opcode(NonZeroU64, u64);

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MachineMode(NonZeroU64, u64);

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Register(NonZeroU64, u64);
