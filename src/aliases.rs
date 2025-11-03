use crate::{
    parser::node::NodeId,
    semantic_analyzer::{scope::Scope, symbol_info::SymbolInfo},
};

pub type Result<T> = std::result::Result<T, crate::err::HarpyError>;
pub type ScopeRc = std::rc::Rc<std::cell::RefCell<Scope>>;
pub type SymbolInfoRef = std::rc::Rc<std::cell::RefCell<SymbolInfo>>;
pub type NodeInfo = std::collections::HashMap<NodeId, SymbolInfoRef>;
