use std::ops::Deref;

use crate::lexer::span::Span;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Node<T> {
    id: NodeId,
    span: Span,
    inner: Box<T>,
}

impl<T> Node<T> {
    pub fn new(id: NodeId, span: Span, inner: T) -> Self {
        Self {
            id,
            span,
            inner: Box::new(inner),
        }
    }

    pub fn extract(self) -> (T, Span) {
        (*self.inner, self.span)
    }

    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Deref for Node<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
