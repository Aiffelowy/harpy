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

    pub fn register(&mut self, lit: Lit) {
        if self.map.contains_key(&lit) {
            return;
        }

        let i = self.pool.len();
        self.pool.push(lit.clone());
        self.map.insert(lit, ConstIndex(i));
    }

    pub fn get(&self, id: ConstIndex) -> Option<&Lit> {
        self.pool.get(id.0)
    }
}
