use std::collections::HashMap;

use crate::{
    compiler::Compiler, intern::Symbol, mach::Register, xva::{XvaBasicBlock, XvaBlockBody, XvaCategory, XvaDest, XvaFunction, XvaOperand, XvaRegister, XvaStatement}
};

#[derive(Default)]
pub struct BlockLocationInfo {
    pub starting_location: Option<Register>,
    pub change_posses: HashMap<usize, Register>,
}

#[derive(Default)]
pub struct BlockLocations {
    pub regs: HashMap<XvaDest, BlockLocationInfo>,
}

struct RegAllocatorState<'a> {
    compiler: &'a dyn Compiler,
    stack_slots: HashMap<XvaDest, i32>,
    map: HashMap<Symbol, BlockLocations>,
    registers_by_kind: HashMap<XvaCategory, &'a [Register]>,
}

impl<'a> RegAllocatorState<'a> {

    fn update_statement(state: &HashMap<XvaDest, Register>, stmt: &mut XvaStatement) -> bool {
        false
    }
    fn update_register_uses(&mut self, block: &mut XvaBasicBlock) -> bool {
        let mut dirty = false;
        let name = block.label;
        let mut current_reg_state = HashMap::new();

        let Some(block_state) = self.map.get_mut(&name) else {
            return false
        };

        for incoming in &mut block.live_at_start {
            match incoming {
                XvaRegister::Virtual(vreg) => {
                    if let Some(reg) = block_state.regs.get(&*vreg).and_then(|v| v.starting_location) {
                        current_reg_state.insert(*vreg, reg);
                        *incoming = XvaRegister::Physical(reg);
                        dirty = true;
                    }
                }
                _ => {}
            }
        }

        match &mut block.body {
            XvaBlockBody::Statement(stmts) => {
                for (n, stmt) in stmts.iter_mut().enumerate() {

                    dirty |= Self::update_statement(&current_reg_state, stmt);

                    for (&reg, loc) in &mut block_state.regs {
                        if let Some(preg) = loc.change_posses.get(&n) {
                            current_reg_state.insert(reg, *preg);
                        }
                    }
                }
            }
        }


        dirty
    }
    fn process_block_phase1(&mut self, block: &mut XvaBasicBlock) -> bool {
        let mut dirty = false;
        let name = block.label;
        let mut back_prop_state = HashMap::new();
        let mut block_locations = self.map.entry(name).or_insert_with(BlockLocations::default);
        match &mut block.body {
            super::XvaBlockBody::Statement(stmts) => {
                for (idx, stmt) in stmts.iter_mut().enumerate().rev() {
                    match stmt {
                        XvaStatement::Jump(next) | XvaStatement::Fallthrough(next) => {
                            if let Some(target_block_locations) = self.map.get(&*next) {
                                for (&reg, loc) in &target_block_locations.regs {
                                    if let Some(start) = loc.starting_location {
                                        back_prop_state.insert(reg, (idx, start));
                                    }
                                }
                            }
                            block_locations = self.map.entry(name).or_insert_with(BlockLocations::default)
                        }
                        super::XvaStatement::Expr(xva_expr) => {
                            let dest = &mut xva_expr.dest;

                            if let XvaRegister::Virtual(v) = dest {
                                if let Some((_, p)) =  back_prop_state.remove(&*v) {
                                    let block_loc_info = block_locations.regs.entry(*v)
                                        .or_insert_with(BlockLocationInfo::default);
                                    block_loc_info.change_posses.insert(idx, p);
                                    *dest = XvaRegister::Physical(p);
                                    dirty = true;
                                }
                            }

                            let left = match &mut xva_expr.op {
                                super::XvaOpcode::Move(reg) => {
                                    reg
                                },
                                super::XvaOpcode::BinaryOp { left, .. } |
                                super::XvaOpcode::CheckedBinaryOp {left, .. } |
                                super::XvaOpcode::UnaryOp { left, .. } => left,
                                _ => continue,
                            };

                            match (&mut *dest, &mut *left) {
                                (XvaRegister::Virtual(v), XvaRegister::Physical(preg)) => {

                                    let block_loc_info = block_locations.regs.entry(*v)
                                        .or_insert_with(BlockLocationInfo::default);

                                        block_loc_info.change_posses
                                        .insert(idx, *preg);

                                    *dest = *left;
                                    dirty = true;
                                }
                                (XvaRegister::Physical(p), XvaRegister::Virtual(v)) => {
                                    back_prop_state.insert(*v, (idx, *p));

                                    *left = *dest;
                                    dirty = true;
                                }
                                _ => {}
                            }

                        },
                        _ => {}
                    }
                }
            },
        }

        for incoming in &mut block.live_at_start {
            if let XvaRegister::Virtual(dest) = incoming {
                if let Some((_, preg)) = back_prop_state.remove(&*dest) {
                    let block_loc_info = block_locations.regs.entry(*dest).or_insert_with(BlockLocationInfo::default);
                    block_loc_info.starting_location = Some(preg);
                }
            }
        }

        dirty
    }
}


pub struct RegAllocator<'a> {
    state: RegAllocatorState<'a>,
    func: &'a mut XvaFunction,
}

impl<'a> RegAllocator<'a> {
    pub fn new(compiler: &'a dyn Compiler, func: &'a mut XvaFunction) -> Self {
        Self {
            state: RegAllocatorState { 
                compiler,
                stack_slots: HashMap::new(),
                map: HashMap::new(),
                registers_by_kind: HashMap::new(),
            },
            func,
        }
    }

    fn process_phase1(&mut self) -> bool {
        let mut dirty = false;
        for block in &mut self.func.body {
            let work = self.state.process_block_phase1(block);
            dirty |= work;

            if work {
                dirty |= self.state.update_register_uses(block);
            }
        }

        dirty
    }

    pub fn process_function(&mut self) {
       
        loop {
            let mut dirty = false;
            loop {
                let mut p1dirty = false;

                p1dirty |= self.process_phase1();
                dirty |= p1dirty;


                if !p1dirty {
                    break
                }
            }

            if !dirty {
                break
            }    
        }
    }
}
