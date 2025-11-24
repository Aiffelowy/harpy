use std::{fmt::Display, ops::Deref};

use crate::{lexer::span::Span, semantic_analyzer::analyze_trait::Analyze};

use super::Parse;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone)]
pub struct Node<T: Parse> {
    id: NodeId,
    span: Span,
    pub(in crate::parser) value: T,
}

impl<T: Parse> Node<T> {
    pub(in crate::parser) fn new(id: NodeId, span: Span, value: T) -> Self {
        Self { id, span, value }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T: Parse> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Parse + Analyze> Analyze for Node<T> {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        self.value.build(builder);
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) -> crate::semantic_analyzer::return_status::ReturnStatus {
        self.value.analyze_semantics(analyzer)
    }
}

impl<T: Parse + Display> Display for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
impl<T: Parse> Node<T> {
    pub fn dummy(node: T) -> Self {
        Self { id: NodeId(0), span: Span::default(), value: node }
    }


    pub fn dummy_with_id(node: T, id: NodeId) -> Self {
        Self { id, span: Span::default(), value: node }
    }
}
