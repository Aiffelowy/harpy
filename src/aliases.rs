use crate::{
    parser::node::NodeId,
    semantic_analyzer::{
        scope::Scope,
        symbol_info::{SymbolInfo, TypeInfo},
    },
};

pub type Result<T> = std::result::Result<T, crate::err::HarpyError>;

pub type ScopeRc = std::rc::Rc<std::cell::RefCell<Scope>>;
pub type SymbolInfoRef = std::rc::Rc<std::cell::RefCell<SymbolInfo>>;
pub type TypeInfoRc = std::rc::Rc<TypeInfo>;

pub type NodeInfo = std::collections::HashMap<NodeId, SymbolInfoRef>;

pub static MAGIC_NUMBER: [u8; 5] = [0x68, 0x61, 0x72, 0x70, 0x79];
pub static VERSION: u16 = 0x1u16;
