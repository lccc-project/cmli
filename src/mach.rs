use crate::{
    compiler::CompilerSpec,
    instr::RegisterKind,
    traits::{AsId, IdType, Name},
    xva::XvaCategory,
};
use std::{hash::Hasher, num::NonZeroU64};

use crate::traits::AsRawId;

#[derive(AsRawId, Copy, Clone, Debug, Hash, PartialEq, Eq, Name)]
pub enum OneMachine {
    Singleton,
}

impl const AsId<MachineMode> for OneMachine {}

pub const ONE_MACHINE: &[MachineMode] = as_id_array!([OneMachine::Singleton] => MachineMode);

pub trait RegisterSpec: AsId<Register> + Name {
    type MachineMode: AsId<MachineMode>;
    /// The Kind of the register
    fn kind(&self) -> RegisterKind;
    /// The size (in bytes) of the register
    fn size(&self, mode: Self::MachineMode) -> u32;

    fn category(&self, mode: Self::MachineMode) -> XvaCategory;

    fn align(&self, mode: Self::MachineMode) -> u32 {
        self.size(mode).next_power_of_two()
    }

    /// Checks whether or not two different registers overlap (IE. refer to different slices of the same register)
    fn overlaps(&self, other: &Self) -> bool;
}

pub trait MachineSpec: Sized {
    type Opcode: AsId<Opcode> + Name;
    const OPCODES: &[Opcode];
    type Register: RegisterSpec<MachineMode = Self::MachineMode>;
    const REGISTERS: &[Register];
    type MachineMode: AsId<MachineMode> + Name;
    const MACH_MODES: &[MachineMode];

    type Compiler: CompilerSpec<Machine = Self>;

    fn name(&self) -> &'static str;

    fn as_compiler(&self) -> &Self::Compiler;
}

mod private {
    pub trait TyOrDefault {
        type Type;
    }
}

use private::TyOrDefault;

impl<T, U> TyOrDefault for (T, U) {
    type Type = U;
}

impl<T> TyOrDefault for (T,) {
    type Type = T;
}

macro_rules! machine_helper {
    (fn $method:ident(&self) -> $ty_name:ident {
        $assoc_const:ident
    } $(impl <$ident:ident> $trait:ident {
        $(fn $fname:ident(&self, $($param:ident: $param_ty:ty),*) -> $ret_ty:ty $block:block)*
    })?) => {
        fn $method(&self) -> <(&(dyn DynList<$ty_name> + '_), $(&(dyn $trait + '_))?) as TyOrDefault>::Type {
            struct ListHelper<M>(core::marker::PhantomData<M>);
            impl<M: MachineSpec> DynList<$ty_name> for ListHelper<M> {
                fn list(&self) -> &[$ty_name] {
                    const { M::$assoc_const }
                }

                fn name_of(&self, val: $ty_name) -> &'static str {
                    match val.downcast::<M::$ty_name>() {
                        Some(val) => val.name(),
                        None => ::core::concat!("**Unknown ", ::core::stringify!($ty_name), "**"),
                    }
                }
            }

            $(impl<$ident: MachineSpec> $trait for ListHelper<$ident> {
                $(fn $fname(&self, $($param: $param_ty),*) -> $ret_ty $block)*
            })?

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
        impl<This> Registers {
            fn register_kind(&self, reg: Register) -> RegisterKind {
                match reg.downcast::<This::Register>() {
                    Some(reg) => reg.kind(),
                    None => panic!("Invalid Register Type"),
                }
            }

            fn register_size(&self, reg: Register, mode: MachineMode) -> u32 {
                match (
                    reg.downcast::<This::Register>(),
                    mode.downcast::<This::MachineMode>(),
                ) {
                    (Some(reg), Some(mode)) => reg.size(mode),
                    (None, _) => panic!("Unknown Register"),
                    (_, None) => panic!("Unknown MachineMode"),
                }
            }

            fn register_align(&self, reg: Register, mode: MachineMode) -> u32 {
                match (
                    reg.downcast::<This::Register>(),
                    mode.downcast::<This::MachineMode>(),
                ) {
                    (Some(reg), Some(mode)) => reg.align(mode),
                    (None, _) => panic!("Unknown Register"),
                    (_, None) => panic!("Unknown MachineMode"),
                }
            }

            fn register_overlaps(&self, reg1: Register, reg2: Register) -> bool {
                match (
                    reg1.downcast::<This::Register>(),
                    reg2.downcast::<This::Register>(),
                ) {
                    (Some(reg1), Some(reg2)) => reg1.overlaps(&reg2),
                    _ => panic!("Unknown Register"),
                }
            }

            fn register_category(&self, reg: Register, mode: MachineMode) -> XvaCategory {
                match (
                    reg.downcast::<This::Register>(),
                    mode.downcast::<This::MachineMode>(),
                ) {
                    (Some(reg), Some(mode)) => reg.category(mode),
                    (None, _) => panic!("Unknown Register"),
                    (_, None) => panic!("Unknown MachineMode"),
                }
            }
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
    fn registers(&self) -> &(dyn Registers + '_);
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

pub trait Registers: DynList<Register> {
    fn register_kind(&self, reg: Register) -> RegisterKind;
    fn register_size(&self, reg: Register, mode: MachineMode) -> u32;
    fn register_align(&self, reg: Register, mode: MachineMode) -> u32;
    fn register_category(&self, reg: Register, mode: MachineMode) -> XvaCategory;

    fn register_overlaps(&self, reg1: Register, reg2: Register) -> bool;
}

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Opcode(NonZeroU64, u64);

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MachineMode(NonZeroU64, u64);

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Register(NonZeroU64, u64);
