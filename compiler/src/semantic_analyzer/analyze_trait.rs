use super::{analyzer::Analyzer, return_status::ReturnStatus, scope_builder::ScopeBuilder};

pub trait Analyze {
    fn build(&self, builder: &mut ScopeBuilder);
    fn analyze_semantics(&self, analyzer: &mut Analyzer) -> ReturnStatus;
}
