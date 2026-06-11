//! Information about machine architectures
//! The base trait of cmli is [`Machine`] from which all features are derived. This trait is dyn-compatible so it can be type-erased
use crate::{
    compiler::CompilerSpec, fmt::{self, PrettyPrinter}, helpers::{Bitset, BitsetIter, BitsetTy}, instr::{Instruction, RegisterKind}, intern::Symbol, traits::{AsId, IdType, IntoId, Name}, xva::XvaCategory
};
use std::{borrow::Borrow, hash::Hasher, iter, num::NonZeroU64, ops::{Deref, DerefMut}};

use crate::traits::AsRawId;


#[derive(AsRawId, Copy, Clone, Debug, Hash, PartialEq, Eq, Name)]
#[repr(u8)]
/// Helper [`MachineMode`] type for machines that do not distinguish between operating modes
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

    fn supported_registers(features: &FeatureSet, mode: Self::MachineMode) -> Regset;
}

pub trait TargetFeatureSpec: Name + Sized {
    fn feature_from_bit(bit: u32) -> Option<Self>;
    fn feature_to_bit(&self) -> u32;
    fn from_name(name: &str) -> Option<Self>;
}

pub trait MachineSpec: Sized {
    type Opcode: AsId<Opcode> + Name;
    const OPCODES: &[Opcode];
    type Register: RegisterSpec<MachineMode = Self::MachineMode>;
    const REGISTERS: &[Register];
    type MachineMode: AsId<MachineMode> + Name + Copy;
    const MACH_MODES: &[MachineMode];

    type TargetFeature: TargetFeatureSpec + Copy;

    type Compiler: CompilerSpec<Machine = Self>;

    fn name(&self) -> &'static str;

    fn as_compiler(&self) -> &Self::Compiler;

    fn pretty_print_size(&self, size: usize) -> Option<&'static str> {
        None
    }

    fn pretty_print_instr(&self, instr: Self::Opcode, mode: Self::MachineMode, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(instr.name())
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

            fn supported_registers(&self, features: &FeatureSet, mode: MachineMode) -> Regset {
                match mode.downcast::<This::MachineMode>() {
                    Some(mode) => {
                        <This as MachineSpec>::Register::supported_registers(features, mode)
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

    fn pretty_print_instr(&self, opc: Opcode, mode: MachineMode, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let instr = opc.downcast().expect("Unknown Opcode");
        let mode = mode.downcast().expect("Unknown Machine Mode");
        <Self as MachineSpec>::pretty_print_instr(&self, instr, mode, f)
    }
    
    fn feature_bit(&self, name: &str) -> u32 {
        let bit = <<Self as MachineSpec>::TargetFeature>::from_name(name).unwrap_or_else(|| panic!("Unknown Target Feature \"{name}\"")).feature_to_bit();

        bit
    }
    
    fn feature_name(&self, bit: u32) -> Option<&'static str> {
        <<Self as MachineSpec>::TargetFeature>::feature_from_bit(bit).map(|n| n.name())
    }
}

pub trait Machine {
    fn name(&self) -> &'static str;
    fn opcodes(&self) -> &(dyn DynList<Opcode> + '_);
    fn registers(&self) -> &(dyn Registers + '_);
    fn modes(&self) -> &(dyn DynList<MachineMode> + '_);
    fn pretty_print_size(&self, size: usize) -> Option<&'static str> {
        None
    }

    fn pretty_print_instr(&self, opc: Opcode, mode: MachineMode, f: &mut core::fmt::Formatter) -> core::fmt::Result;

    fn feature_bit(&self, name: &str) -> u32;

    fn feature_name(&self, bit: u32) -> Option<&'static str>;
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

    fn supported_registers(&self, features: &FeatureSet, mode: MachineMode) -> Regset;
}

#[derive(IdType, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Opcode(NonZeroU64, u64);

impl<'a> core::fmt::Display for PrettyPrinter<'a, Opcode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.pretty_print_instr(*self.0, self.2, f)
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

const REGSET_SIZE: usize = 8;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Regset(Bitset<RegsetBit, REGSET_SIZE>);

impl const Deref for Regset {
    type Target = Bitset<RegsetBit, REGSET_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl const DerefMut for Regset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Regset {
    pub const fn new() -> Self {
        Self(Bitset::new())
    }

    pub fn from_regids<R: IntoId<Register>>(iter: impl IntoIterator<Item = R>, mach: &dyn Machine) -> Self {
        let mut array = Self::new();

        array.insert_regids(iter, mach);

        array
    }

    pub fn from_registers<R: RegisterSpec>(iter: impl IntoIterator<Item = R>) -> Self {
        let mut array = Self::new();

        array.insert_registers(iter);

        array
    }

    pub fn insert_regid<R: IntoId<Register>>(&mut self, reg: R, mach: &dyn Machine) {
        let reg = reg.into_id();

        let bit = mach.registers().regmap_bit(reg).expect("Cannot push register");

        self.insert_bit(RegsetBit(bit))
    }

    pub fn insert_regids<R: IntoId<Register>>(&mut self, iter: impl IntoIterator<Item = R>, mach: &dyn Machine) {
        for reg in iter {
            self.insert_regid(reg, mach);
        }
    }

    pub fn remove_regid<R: IntoId<Register>>(&mut self, reg: R, mach: &dyn Machine) {
        let reg = reg.into_id();

        let bit = mach.registers().regmap_bit(reg).expect("Cannot push register");

        self.remove_bit(RegsetBit(bit))
    }

    pub fn remove_regids<R: IntoId<Register>>(&mut self, iter: impl IntoIterator<Item = R>, mach: &dyn Machine) {
        for reg in iter {
            self.remove_regid(reg, mach);
        }
    }

    pub fn contains_regid<R: IntoId<Register>>(&self, reg: R, mach: &dyn Machine) -> bool {
        let reg = reg.into_id();

        let bit = mach.registers().regmap_bit(reg).expect("Cannot push register");

        self.contains_bit(RegsetBit(bit))
    }

    pub fn contains_any_regids<R: IntoId<Register>>(&self, reg: impl IntoIterator<Item = R>, mach: &dyn Machine) -> bool {
        reg.into_iter().any(|r| self.contains_regid(r, mach))
    }

    pub fn contains_all_regids<R: IntoId<Register>>(&self, reg: impl IntoIterator<Item = R>, mach: &dyn Machine) -> bool {
        reg.into_iter().all(|r| self.contains_regid(r, mach))
    }

    pub fn into_regids<'a>(self, mach: &'a dyn Machine, mode: MachineMode) -> RegsetIntoRegisters<'a> {
        RegsetIntoRegisters(self.into_iter(), mach.registers(), mode)
    }

    pub fn retain_all_regids<R: IntoId<Register>>(&mut self, reg: impl IntoIterator<Item = R>, mach: &dyn Machine) {
        let other = Self::from_regids(reg, mach);

        self.retain_mask(*other);
    }

    pub fn insert_register<R: RegisterSpec>(&mut self, r: R) {
        let bit = r.regmap_bit().expect("Cannot Insert Register");

        self.insert_bit(RegsetBit(bit));
    }
    pub fn remove_register<R: RegisterSpec>(&mut self, r: R) {
        let bit = r.regmap_bit().expect("Cannot remove Register");

        self.remove_bit(RegsetBit(bit));
    }

    pub fn contains_register<R: RegisterSpec>(&self, r: R) -> bool {
        let bit = r.regmap_bit().expect("Cannot remove Register");

        self.contains_bit(RegsetBit(bit))
    }

    pub fn insert_registers<R: RegisterSpec, I: IntoIterator<Item = R>>(&mut self, regs: I) {
        for reg in regs {
            self.insert_register(reg)
        }
    }

    pub fn remove_registers<R: RegisterSpec, I: IntoIterator<Item = R>>(&mut self, regs: I) {
        for reg in regs {
            self.remove_register(reg)
        }
    }

    pub fn contains_all_registers<R: RegisterSpec, I: IntoIterator<Item = R>>(&self, regs: I) -> bool {
        regs.into_iter().all(|r| self.contains_register(r))
    }

    pub fn contains_any_registers<R: RegisterSpec, I: IntoIterator<Item = R>>(&self, regs: I) -> bool {
        regs.into_iter().all(|r| self.contains_register(r))
    }
}

impl FromIterator<RegsetBit> for Regset {
    fn from_iter<T: IntoIterator<Item = RegsetBit>>(iter: T) -> Self {
        Regset(Bitset::from_iter(iter))
    }
}

impl Extend<RegsetBit> for Regset {
    fn extend<T: IntoIterator<Item = RegsetBit>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl IntoIterator for Regset {
    type Item = RegsetBit;
    type IntoIter = BitsetIter<RegsetBit, REGSET_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, Regset> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::pretty_print_list(*self.0, ", ", self.1, self.2).fmt(f)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RegsetBit(u32);

impl const BitsetTy for RegsetBit {
    fn from_u32(val: u32) -> Self {
        Self(val)
    }
    fn into_u32(self) -> u32 {
        self.0
    }
}

pub struct RegsetIntoRegisters<'a>(BitsetIter<RegsetBit, REGSET_SIZE>, &'a dyn Registers, MachineMode);

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
            None => f.write_fmt(format_args!("/*UNKNOWN REGISTER {:#04X}*/", self.0.0)),
        }
    }
}


#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FeatureBit(u32);

impl const BitsetTy for FeatureBit {
    fn from_u32(bit: u32) -> Self {
        Self(bit)
    }

    fn into_u32(self) -> u32 {
        self.0
    }
}

impl<'a> core::fmt::Display for PrettyPrinter<'a, FeatureBit> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(name) = self.1.feature_name(self.0.0) else {
            return f.write_fmt(format_args!("/*UNKNOWN FEATURE {:02X}*/", self.0.0))
        };

        f.write_str("\"")?;
        f.write_str(name)?;
        f.write_str("\"")
    }
}

const FEATURESET_SIZE: usize = 4;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FeatureSet(Bitset<FeatureBit, FEATURESET_SIZE>);

impl Default for FeatureSet {
    fn default() -> Self {
        Self::new()
    }
}

impl const Deref for FeatureSet {
    type Target = Bitset<FeatureBit, FEATURESET_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl const DerefMut for FeatureSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FeatureSet {
    pub const fn new() -> Self {
        Self(Bitset::new())
    }

    pub fn from_names<S: AsRef<str>, I: IntoIterator<Item = S>>(iter: I, mach: &dyn Machine) -> Self {
        iter.into_iter().map(|s| mach.feature_bit(s.as_ref())).map(FeatureBit).collect()
    }

    pub fn insert_names<S: AsRef<str>, I: IntoIterator<Item = S>>(&mut self, iter: I, mach: &dyn Machine) {
        for name in iter {
            self.insert_name(name.as_ref(), mach)
        }
    }

    pub fn insert_name(&mut self, name: &str, mach: &dyn Machine) {
        let bit = mach.feature_bit(name);

        self.insert_bit(FeatureBit(bit));
    }

    pub fn contains_name(&self, name: &str, mach: &dyn Machine) -> bool {
        let bit = mach.feature_bit(name);

        self.contains_bit(FeatureBit(bit))
    }

    pub fn remove_name(&mut self, name: &str, mach: &dyn Machine) {
        let bit = mach.feature_bit(name);

        self.remove_bit(FeatureBit(bit));
    }

    pub fn contains_all_names<S: AsRef<str>, I: IntoIterator<Item = S>>(&self, iter: I, mach: &dyn Machine) -> bool {
        iter.into_iter().all(|n| self.contains_name(n.as_ref(), mach))
    }

    pub fn contains_any_names<S: AsRef<str>, I: IntoIterator<Item = S>>(&self, iter: I, mach: &dyn Machine) -> bool {
        iter.into_iter().any(|n| self.contains_name(n.as_ref(), mach))
    }

    pub fn remove_names<S: AsRef<str>, I: IntoIterator<Item = S>>(&mut self, iter: I, mach: &dyn Machine) {
        for item in iter {
            self.remove_name(item.as_ref(), mach)
        }
    }

    pub fn insert_feature<F: TargetFeatureSpec>(&mut self, feat: &F) {
        let bit = feat.feature_to_bit();

        self.insert_bit(FeatureBit(bit));
    }

    pub fn remove_feature<F: TargetFeatureSpec>(&mut self, feat: &F) {
        let bit = feat.feature_to_bit();

        self.remove_bit(FeatureBit(bit));
    }

    pub fn contains_feature<F: TargetFeatureSpec>(&self, feat: &F) -> bool {
        let bit = feat.feature_to_bit();

        self.contains_bit(FeatureBit(bit))
    } 

    pub fn remove_features<F: TargetFeatureSpec, I: IntoIterator<Item: Borrow<F>>>(&mut self, iter: I) {
        for feat in iter {
            self.remove_feature(feat.borrow())
        }
    }

    pub fn contains_all_features<F: TargetFeatureSpec, I: IntoIterator<Item: Borrow<F>>>(&self, iter: I) -> bool {
        iter.into_iter().all(|f| self.contains_feature(f.borrow()))
    }

    pub fn contains_any_features<F: TargetFeatureSpec, I: IntoIterator<Item: Borrow<F>>>(&self, iter: I) -> bool {
        iter.into_iter().any(|f| self.contains_feature(f.borrow()))
    }
}

impl<F: TargetFeatureSpec> FromIterator<F> for FeatureSet {
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        iter.into_iter().map(|f| f.feature_to_bit()).map(FeatureBit).collect()
    }
}

impl FromIterator<FeatureBit> for FeatureSet {
    fn from_iter<T: IntoIterator<Item = FeatureBit>>(iter: T) -> Self {
        Self(Bitset::from_iter(iter))
    }
}

impl<F: TargetFeatureSpec> Extend<F> for FeatureSet {
    fn extend<T: IntoIterator<Item = F>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|f| f.feature_to_bit()).map(FeatureBit))
    }
}

impl Extend<FeatureBit> for FeatureSet {
    fn extend<T: IntoIterator<Item = FeatureBit>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl core::fmt::Display for PrettyPrinter<'_, FeatureSet> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        PrettyPrinter(&self.0.0, self.1, self.2).fmt(f)
    }
}