use crate::aliases::Result;

use super::{analyzer::Analyzer, scope_builder::ScopeBuilder};

pub trait Analyze {
    fn build(&self, builder: &mut ScopeBuilder) -> Result<()>;
    fn analyze_semantics(&self, analyzer: &mut Analyzer) -> Result<()>;
}
