use std::collections::HashMap;

use crate::analyzer::tables::{
    function_table::FunctionId, global_table::GlobalId, struct_table::StructId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub usize);

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub parent: Option<ModuleId>,

    pub sub_modules: HashMap<String, ModuleId>,
    pub structs: HashMap<String, StructId>,
    pub functions: HashMap<String, FunctionId>,
    pub globals: HashMap<String, GlobalId>,
}

impl Module {
    pub fn new(name: impl Into<String>, parent: Option<ModuleId>) -> Self {
        Self {
            name: name.into(),
            parent,

            sub_modules: HashMap::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            globals: HashMap::new(),
        }
    }
}
