use std::collections::HashMap;

use crate::lexer::tokens::Lit;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstIndex(pub usize);

#[derive(Debug)]
pub struct ConstPool {
    pool: Vec<Lit>,
    map: HashMap<Lit, ConstIndex>,
}

impl ConstPool {
    pub(in crate::semantic_analyzer) fn new() -> Self {
        Self {
            pool: vec![],
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, lit: Lit) -> ConstIndex {
        if let Some(i) = self.map.get(&lit) {
            return *i;
        }

        let i = ConstIndex(self.pool.len());
        self.pool.push(lit.clone());
        self.map.insert(lit, i);
        i
    }

    pub fn get(&self, id: ConstIndex) -> Option<&Lit> {
        self.pool.get(id.0)
    }
}
