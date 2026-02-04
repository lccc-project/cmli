use std::collections::HashMap;

use crate::intern::Symbol;

#[derive(Clone, Debug)]
pub struct TargetInfo {
    pub properties: TargetProperties,
    pub ptr_width: u16,
}

#[derive(Clone, Debug)]
pub struct TargetProperties {
    pub global_properties: HashMap<Symbol, PropertyValue>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PropertyValue {
    String(Symbol),
    Int(i64),
    Bool(bool),
}
