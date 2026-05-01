use std::ops::Deref;

use crate::lexer::span::Span;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
    pub id: NodeId,
    pub span: Span,
    pub inner: T,
}

impl<T> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
