use std::{collections::HashMap, fmt::Display};

use crate::{
    analyzer::{
        analyzer::{Analyzer, Fallback},
        types::types::TypeId,
    },
    lexer::span::Span,
    parser::{
        expr::expr_defs::FunctionArg,
        node::NodeId,
        stmt::stmts::{GlobalStmt, LetStmt},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

impl SymbolId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}
impl Fallback<SymbolId> for Analyzer {
    fn fallback(&self) -> SymbolId {
        SymbolId(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Local,
    Parameter,
    Global,
    Function,
    Struct,
    Dummy,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: TypeId,
    pub is_mutable: bool,
    pub kind: SymbolKind,
    pub declared_at: Span,
}

impl Symbol {
    pub fn from_let(stmt: &LetStmt, ty: TypeId) -> Self {
        Self {
            name: stmt.name.value().clone(),
            ty,
            is_mutable: stmt.mutable,
            declared_at: stmt.name.span(),
            kind: SymbolKind::Local,
        }
    }

    pub fn from_global(stmt: &GlobalStmt, ty: TypeId) -> Self {
        Self {
            name: stmt.name.value().clone(),
            ty,
            is_mutable: stmt.mutable,
            declared_at: stmt.name.span(),
            kind: SymbolKind::Global,
        }
    }

    pub fn from_arg(arg: &FunctionArg, ty: TypeId) -> Self {
        Self {
            name: arg.name.value().clone(),
            ty,
            is_mutable: arg.mutable,
            declared_at: arg.name.span(),
            kind: SymbolKind::Parameter,
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub resolutions: HashMap<NodeId, SymbolId>,
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.symbols.is_empty() {
            return writeln!(f, "  <No symbols defined>");
        }

        writeln!(
            f,
            "  ID         | Name                 | Type       | Mutable"
        )?;
        writeln!(
            f,
            "  -----------+----------------------+------------+---------"
        )?;

        for sym in &self.symbols {
            let ty_str = format!("{:?}", sym.ty);

            let mut_str = if sym.is_mutable { "Yes" } else { "No " };

            writeln!(f, "{:<20} | {:<10} | {}", sym.name, ty_str, mut_str)?;
        }

        writeln!(
            f,
            "  -----------+----------------------+------------+---------"
        )?;
        writeln!(
            f,
            "  * Resolution Stats: {} AST nodes successfully bound to symbols.",
            self.resolutions.len()
        )?;

        Ok(())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        let mut s = Self {
            symbols: Vec::new(),
            resolutions: HashMap::new(),
        };

        let dummy = Symbol {
            name: "<unknown_symbol>".to_owned(),
            ty: TypeId(0),
            is_mutable: false,
            declared_at: Span::dummy(),
            kind: SymbolKind::Dummy,
        };
        s.register(dummy);
        s
    }
}

impl SymbolTable {
    pub fn register(&mut self, def: Symbol) -> SymbolId {
        let id = SymbolId(self.symbols.len());
        self.symbols.push(def);
        id
    }

    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id.0]
    }

    pub fn get_symbol(&self, id: NodeId) -> &Symbol {
        let Some(id) = self.resolutions.get(&id) else {
            return &self.symbols[0];
        };
        &self.symbols[id.0]
    }

    pub fn get_symbol_mut(&mut self, id: NodeId) -> Option<&mut Symbol> {
        let id = self.resolutions.get(&id)?;
        Some(&mut self.symbols[id.0])
    }

    pub fn add_resolution(&mut self, node_id: NodeId, symbol_id: SymbolId) {
        self.resolutions.insert(node_id, symbol_id);
    }
}
