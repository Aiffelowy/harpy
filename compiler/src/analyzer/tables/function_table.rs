use std::{collections::HashMap, fmt::Display};

use crate::{
    analyzer::{
        analyzer::{Analyzer, Fallback},
        tables::symbol_table::SymbolId,
        types::types::TypeId,
    },
    lexer::span::Span,
    parser::node::NodeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

impl FunctionId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    pub fn get<'a>(&'a self, analyzer: &'a Analyzer) -> &'a FunctionDef {
        analyzer.db.function_table.get(*self)
    }
    pub fn get_mut<'a>(&'a self, analyzer: &'a mut Analyzer) -> &'a mut FunctionDef {
        analyzer.db.function_table.get_mut(*self)
    }
}

impl Fallback<FunctionId> for Analyzer {
    fn fallback(&self) -> FunctionId {
        FunctionId(0)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub id: Option<FunctionId>,
    pub signature: TypeId,
    pub params: Vec<SymbolId>,
    pub locals: Vec<SymbolId>,
    pub span: Span,
}

impl FunctionDef {
    pub fn skeleton(name: String, signature: TypeId, span: Span) -> Self {
        FunctionDef {
            name,
            id: None,
            signature,
            params: Vec::new(),
            span,
            locals: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionTable {
    pub functions: Vec<FunctionDef>,
    pub resolutions: HashMap<NodeId, FunctionId>,
}

impl Display for FunctionTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.functions.is_empty() {
            return writeln!(f, "  <No functions defined>");
        }

        writeln!(
            f,
            "  ID         | Name                 | Signature  | Params               | Locals"
        )?;
        writeln!(f, "  -----------+----------------------+------------+----------------------+----------------------")?;

        for func in &self.functions {
            let id_str = match &func.id {
                Some(id) => format!("{:?}", id),
                None => "[Unset]".to_string(),
            };

            let sig_str = format!("{:?}", func.signature);

            let params_str = if func.params.is_empty() {
                "[]".to_string()
            } else {
                format!("{:?}", func.params)
            };

            let locals_str = if func.locals.is_empty() {
                "[]".to_string()
            } else {
                format!("{:?}", func.locals)
            };

            writeln!(
                f,
                "  {:<10} | {:<20} | {:<10} | {:<20} | {}",
                id_str, func.name, sig_str, params_str, locals_str
            )?;
        }

        Ok(())
    }
}

impl Default for FunctionTable {
    fn default() -> Self {
        let mut s = Self {
            functions: Vec::new(),
            resolutions: HashMap::new(),
        };
        let dummy = FunctionDef::skeleton("<unknown_fn>".to_owned(), TypeId(0), Span::dummy());
        s.register(dummy);
        s
    }
}

impl FunctionTable {
    pub fn register(&mut self, mut def: FunctionDef) -> FunctionId {
        let id = FunctionId(self.functions.len());
        def.id = Some(id);
        self.functions.push(def);

        id
    }

    pub fn add_resolution(&mut self, node_id: NodeId, fn_id: FunctionId) {
        self.resolutions.insert(node_id, fn_id);
    }

    pub fn get_resolution(&mut self, node_id: NodeId) -> FunctionId {
        *self.resolutions.get(&node_id).unwrap_or(&FunctionId(0))
    }

    pub fn get(&self, id: FunctionId) -> &FunctionDef {
        &self.functions[id.0]
    }

    pub fn get_mut(&mut self, id: FunctionId) -> &mut FunctionDef {
        &mut self.functions[id.0]
    }
}
