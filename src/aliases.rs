use crate::{
    lexer::tokens::Literal, parser::{node::NodeId, types::Type}, semantic_analyzer::{
        scope::Scope,
        symbol_info::{SymbolInfo, TypeInfo},
    }
};

pub type Result<T> = std::result::Result<T, crate::err::HarpyError>;

pub type ScopeRc = std::rc::Rc<std::cell::RefCell<Scope>>;
pub type SymbolInfoRef = std::rc::Rc<std::cell::RefCell<SymbolInfo>>;
pub type TypeInfoRc = std::rc::Rc<TypeInfo>;

pub type NodeInfo = std::collections::HashMap<NodeId, SymbolInfoRef>;
pub type TypeInfos = std::collections::HashMap<Type, TypeInfoRc>;
pub type ConstInfo = std::collections::HashMap<,Literal>;
