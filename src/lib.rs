#![feature(never_type, macro_metavar_expr)]

//!
//! ## Features
//! * `error-track-caller`: This allows extended debugging via the [`error::Error`] body by tracking the constructor source of errors.
//! * `debug-error-track-caller`: Same as `error-track-caller` but only when debug assertions are enabled (note that this uses the setting for this crate, not the top level binary)
//! Note that if both this and `error-track-caller` is on, `error-track-caller` prevails and constructor locations are tracked in all cases.
//! * `default-archs`: Enables architectures marked "default"
//! * `all-archs`: Enables all arch features
//! * `all-timings`: Enables timings for all enabled archs that supports it.
//!
//! ### Architectures
//! The following architectures are supported, and may support timings (enabled by either `arch-timings` or `<arch>-timings` feature):
//! * `x86` (timings: yes, default: yes)

pub mod arch;
pub mod asm;
pub mod compiler;
pub mod encode;
pub mod error;
pub mod instr;
pub mod intern;
pub mod traits;
pub mod xva;

pub use error::Result;

extern crate self as cmli;

mod helpers;
