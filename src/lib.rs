#![feature(
    macro_derive,
    macro_metavar_expr,
    const_trait_impl,
    const_cmp,
    adt_const_params
)]

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
