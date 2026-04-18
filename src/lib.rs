//! # Compiler Machine Later Interface
//! cmli is a low level generic description of Machine features used as the final stage of compilation before machine code/object files are created.
//! 
//! Two parts of CMLI are defined:
//! * Machine Definition: Provides a descriptive interface for defining architectures and various support (such as [`compiler`] and [`asm`]), and
//! * XVA (Extensible Virtual Architecture): A Very Low Level 3AC IR for representing the final stages of high-level compilation and the conversion into raw machine code
//! 
//! ## XVA
//! 
//! XVA Depends on the Machine definition (namely, it depends on an architecutre supporting [`Compiler`][compiler::Compiler]). 
//! It supports low-level optimization, is responsible for register allocation, and Machine Code Emission.
//! It does not support complex values or types. See the [`xva`] module for more details
//! 
//! ## Features
//! The features of the crate mostly control the architecture or object formats supported by the crate.
//! 
//! By default, all default architectures and formats are enabled
//! 
//! Architecture Features:
//! * x86: Supports x86 (16-bit, 32-bit, or 64-bit) architecture, compilation, and assembly,
//! * default-archs (default): Enables most common architectures supported by cmli (currently `x86`)
//! * all-archs: Enables all architectures supported by cmli
//!

#![feature(
    macro_derive,
    macro_metavar_expr,
    const_trait_impl,
    const_cmp,
    adt_const_params
)]
// #![deny(missing_docs)]

#[macro_use]
#[doc(hidden)]
pub mod macros;

pub mod asm;
pub mod compiler;
pub mod instr;
pub mod intern;
pub mod mach;
pub mod mem;
pub mod target;
pub mod traits;

pub mod xva;

pub mod archs;

pub mod fmt;
