use std::collections::HashMap;

use crate::{
    mach::Register,
    xva::{XvaCategory, XvaRegister},
};

pub struct LocationInfo {
    pub stack_slot: Option<i32>,
    pub assign_pos: HashMap<u32, Register>,
}

pub struct RegAllocator {
    map: HashMap<XvaRegister, LocationInfo>,
    registers_by_kind: HashMap<XvaCategory, Vec<Register>>,
}
