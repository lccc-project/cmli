//! Information about machine architectures
//! The base trait of cmli is [`Machine`] from which all features are derived. This trait is dyn-compatible so it can be type-erased
use crate::{
    compiler::CompilerSpec,
    fmt::{self, PrettyPrinter},
    instr::{Instruction, RegisterKind},
    traits::{AsId, IdType, IntoId, Name},
    xva::XvaCategory,
};
use std::{hash::Hasher, iter, num::NonZeroU64};

use crate::traits::AsRawId;

/// Helper [`MachineMode`] type for machines that do not distinguish between operating modes
#[derive(AsRawId, Copy, Clone, Debug, Hash, PartialEq, Eq, Name)]
pub enum OneMachine {
    /// Singleton instance of [`OneMachine`]
    Singleton,
}

impl const AsId<MachineMode> for OneMachine {}

/// Array of [`MachineMode`] values containing solely [`OneMachine::Singleton`]
pub const ONE_MACHINE: &[MachineMode] = as_id_array!([OneMachine::Singleton] => MachineMode);

/// Specification trait for providing information about CPU registers
/// Can be combined with [`MachineSpec`] to implement the [`Registers`] trait
pub trait RegisterSpec: AsId<Register> + Name + Sized {
    /// The machine modes supported by this register kind. This should match the corresponding [`MachineSpec`]
    type MachineMode: AsId<MachineMode>;
    /// The Kind of the register
    fn kind(&self) -> RegisterKind;
    /// The size (in bytes) of the register in `mode`.
    fn size(&self, mode: Self::MachineMode) -> u32;

    /// Determines the category of the specified register in the current `mode`
    fn category(&self, mode: Self::MachineMode) -> XvaCategory;

    /// Determines the optimistic alignment requirement for the physical register in the current `mode`
    fn align(&self, mode: Self::MachineMode) -> u32 {
        self.size(mode).next_power_of_two()
    }

    /// Checks whether or not two different registers overlap (IE. refer to different slices of the same register)
    fn overlaps(&self, other: &Self) -> bool;

    fn from_bit(bit: u32, mode: Self::MachineMode) -> Option<Self>;

    fn regmap_bit(self) -> Option<u32>;
}

pub trait MachineSpec: Sized {
    type Opcode: AsId<Opcode> + Name;
    const OPCODES: &[Opcode];
    type Register: RegisterSpec<MachineMode = Self::MachineMode>;
    const REGISTERS: &[Register];
    type MachineMode: AsId<MachineMode> + Name + Copy;
    const MACH_MODES: &[MachineMode];

    type Compiler: CompilerSpec<Machine = Self>;

    fn name(&self) -> &'static str;

    fn as_compiler(&self) -> &Self::Compiler;

    fn pretty_print_size(&self, size: usize) -> Option<&'static str> {
        None
    }
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

    fn pretty_print_size(&self, size: usize) -> Option<&'static str> {
        <Self as MachineSpec>::pretty_print_size(self, size)
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

            fn regmap_bit(&self, reg: Register) -> Option<u32> {
                match reg.downcast::<This::Register>() {
                    Some(reg) => reg.regmap_bit(),
                    None => panic!("Unknown Register"),
                }
            }
            fn regmap_from_bit(&self, bit: u32, mode: MachineMode) -> Option<Register> {
                match mode.downcast::<This::MachineMode>() {
                    Some(mode) => {
                        <This as MachineSpec>::Register::from_bit(bit, mode).map(Register::new)
                    }
                    None => panic!("Unknown MachineMode"),
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
    fn pretty_print_size(&self, size: usize) -> Option<&'static str> {
        None
    }
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

    fn regmap_bit(&self, reg: Register) -> Option<u32>;
    fn regmap_from_bit(&self, bit: u32, mode: MachineMode) -> Option<Register>;

    fn register_overlaps(&self, reg1: Register, reg2: Register) -> bool;
}

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Opcode(NonZeroU64, u64);

impl<'a> core::fmt::Display for PrettyPrinter<'a, Opcode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.1.opcodes().name_of(*self.0))
    }
}

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MachineMode(NonZeroU64, u64);

impl<'a> core::fmt::Display for PrettyPrinter<'a, MachineMode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.1.modes().name_of(*self.0))
    }
}

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Register(NonZeroU64, u64);

impl<'a> core::fmt::Display for PrettyPrinter<'a, Register> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.1.registers().name_of(*self.0))
    }
}

const REGSET_SIZE: usize = 4;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Regset([u64; REGSET_SIZE]);

impl Regset {
    pub const fn new() -> Self {
        Self([0; REGSET_SIZE])
    }

    pub fn from_registers<R: IntoId<Register>>(iter: impl IntoIterator<Item = R>, mach: &dyn Machine) -> Self {
        let mut array = Self::new();

        array.insert_registers(iter, mach);

        array
    }

    pub const fn insert_bit(&mut self, bit: RegsetBit) {
        let idx = (bit.0 >> 6) as usize;
        self.0[idx] |= (1 << (bit.0 & 63));
    }

    pub fn insert_register<R: IntoId<Register>>(&mut self, reg: R, mach: &dyn Machine) {
        let reg = reg.into_id();

        let bit = mach.registers().regmap_bit(reg).expect("Cannot push register");

        self.insert_bit(RegsetBit(bit))
    }

    pub fn insert_registers<R: IntoId<Register>>(&mut self, iter: impl IntoIterator<Item = R>, mach: &dyn Machine) {
        for reg in iter {
            self.insert_register(reg, mach);
        }
    }

    pub const fn remove_bit(&mut self, bit: RegsetBit) {
        let idx = (bit.0 >> 6) as usize;
        self.0[idx] &= !(1 << (bit.0 & 63));
    }

    pub fn remove_register<R: IntoId<Register>>(&mut self, reg: R, mach: &dyn Machine) {
        let reg = reg.into_id();

        let bit = mach.registers().regmap_bit(reg).expect("Cannot push register");

        self.remove_bit(RegsetBit(bit))
    }

    pub fn remove_registers<R: IntoId<Register>>(&mut self, iter: impl IntoIterator<Item = R>, mach: &dyn Machine) {
        for reg in iter {
            self.remove_register(reg, mach);
        }
    }

    pub const fn contains_bit(&self, bit: RegsetBit) -> bool {
        let idx = (bit.0 >> 6) as usize;

        (self.0[idx] & (1 << (bit.0 & 63))) != 0
    }

    pub fn contains_register<R: IntoId<Register>>(&self, reg: R, mach: &dyn Machine) -> bool {
        let reg = reg.into_id();

        let bit = mach.registers().regmap_bit(reg).expect("Cannot push register");

        self.contains_bit(RegsetBit(bit))
    }

    pub fn contains_any<R: IntoId<Register>>(&self, reg: impl IntoIterator<Item = R>, mach: &dyn Machine) -> bool {
        reg.into_iter().any(|r| self.contains_register(r, mach))
    }

    pub fn contains_all<R: IntoId<Register>>(&self, reg: impl IntoIterator<Item = R>, mach: &dyn Machine) -> bool {
        reg.into_iter().all(|r| self.contains_register(r, mach))
    }

    pub fn into_registers<'a>(self, mach: &'a dyn Machine, mode: MachineMode) -> RegsetIntoRegisters<'a> {
        RegsetIntoRegisters(self.into_iter(), mach.registers(), mode)
    }

    pub fn retain_all<R: IntoId<Register>>(&mut self, reg: impl IntoIterator<Item = R>, mach: &dyn Machine) {
        let other = Self::from_registers(reg, mach);

        self.retain_mask(other);
    }

    pub fn retain_mask(&mut self, other: Regset) {
        for (a, b) in iter::zip(&mut self.0, other.0) {
            *a &= b;
        }
    }

    pub fn remove_mask(&mut self, other: Regset) {
        for (a, b) in iter::zip(&mut self.0, other.0) {
            *a &= !b;
        }
    }
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, Regset> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::pretty_print_list(*self.0, ", ", self.1, self.2).fmt(f)
    }
}

impl FromIterator<RegsetBit> for Regset {
    fn from_iter<T: IntoIterator<Item = RegsetBit>>(iter: T) -> Self {
        let mut v = const { Regset([0; REGSET_SIZE]) };
        v.extend(iter);
        v
    }
}

impl Extend<RegsetBit> for Regset {
    fn extend<T: IntoIterator<Item = RegsetBit>>(&mut self, iter: T) {
        for bit in iter {
            self.insert_bit(bit);
        }
    }
}

impl IntoIterator for Regset {
    type Item = RegsetBit;
    type IntoIter = RegsetIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = self.0.into_iter();

        let val = iter.next().unwrap();
        RegsetIter(iter, val, 0)
    }
}

pub struct RegsetIter(core::array::IntoIter<u64, REGSET_SIZE>, u64, u32);

impl Iterator for RegsetIter {
    type Item = RegsetBit;

    fn next(&mut self) -> Option<Self::Item> {
        while self.1 == 0 {
            self.2 = (self.2 & 63) + 64;
            self.1 = self.0.next()?;
        }

        let p = self.1.trailing_zeros();
        self.1 >>= p + 1;
        let val = self.2 + p;
        self.2 += p + 1;

        Some(RegsetBit(val))
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RegsetBit(u32);

pub struct RegsetIntoRegisters<'a>(RegsetIter, &'a dyn Registers, MachineMode);

impl<'a> Iterator for RegsetIntoRegisters<'a> {
    type Item = Register;

    fn next(&mut self) -> Option<Self::Item> {
        let bit = self.0.next()?;

        self.1.regmap_from_bit(bit.0, self.2)
    }
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, RegsetBit> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1.registers().regmap_from_bit(self.0.0, self.2) {
            Some(reg) => f.write_str(self.1.registers().name_of(reg)),
            None => f.write_str("/*UNKNOWN REGISTER*/"),
        }
    }
}
