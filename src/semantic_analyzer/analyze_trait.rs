use super::{analyzer::Analyzer, scope_builder::ScopeBuilder};

pub trait Analyze {
    fn build(&self, builder: &mut ScopeBuilder);
    fn analyze_semantics(&self, analyzer: &mut Analyzer);
}
