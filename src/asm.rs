//! Assembly Support

use std::num::NonZero;

use crate::mach::{DynList, Machine};

use crate::traits::{AsId, Name};
use crate::{IdType, AsRawId};

#[cfg(feature = "asm-parse")]
pub mod parser;

#[cfg(feature = "asm-print")]
pub mod printer;


#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, IdType)]
pub struct AsmSyntax(NonZero<u64>, u64);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, AsRawId)]
pub enum OneSyntax {
    Default,
}

impl Name for OneSyntax {
    fn name(&self) -> &'static str {
        "default"
    }
}

const impl AsId<AsmSyntax> for OneSyntax{}

pub trait AsmInfo {
    fn machine(&self) -> &dyn Machine;

    fn syntax_list(&self) -> &dyn DynList<AsmSyntax>;

    fn default_syntax(&self) -> AsmSyntax;

    #[cfg(feature = "asm-parse")]
    fn parser(&self) -> Option<&dyn parser::AsmParser> {
        None
    }

    #[cfg(feature = "asm-print")]
    fn printer(&self) -> Option<&dyn printer::AsmPrinter> {
        None
    }
}