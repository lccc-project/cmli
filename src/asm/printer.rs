use crate::{asm::{AsmInfo, AsmSyntax}, instr::Instruction, mach::MachineMode};


pub trait AsmPrinter {
    fn asm_info(&self) -> &dyn AsmInfo;

    fn print_instruction(&self,f : &mut dyn core::fmt::Write, instr: Instruction,  syntax: AsmSyntax) -> core::fmt::Result;

    fn print_mode_directive(&self, f: &mut dyn core::fmt::Write, new_mode: MachineMode) -> core::fmt::Result;

    fn print_syntax_directive(&self, f: &mut dyn core::fmt::Write, syntax: AsmSyntax) -> core::fmt::Result;
}