use crate::instr_set;

pub enum Width {
    Bits(u16),
    Native,
}

use Width::*;
use with_builtin_macros::with_builtin;

with_builtin! {
    let $base = include_from_root!("arch/x86/base.ainfo") in {
        instr_set!{$base}
    }
}

with_builtin! {
    let $mach = include_from_root!("arch/x86/mach.ainfo") in {
        instr_set!{$mach}
    }
}
