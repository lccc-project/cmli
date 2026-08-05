use crate::mach::Machine;

pub trait AsmParser {
    fn as_machine(&self) -> &dyn Machine;
}
