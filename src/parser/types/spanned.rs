use std::{fmt::Display, ops::Deref};

use crate::{
    aliases::Result,
    lexer::span::Span,
    parser::{parser::Parser, Parse},
};

use super::Type;

#[derive(Debug, Clone)]
pub struct TypeSpanned {
    pub ty: Type,
    pub span: Span,
}

impl Parse for TypeSpanned {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let (ty, span) = parser.parse_spanned()?;
        Ok(Self { ty, span })
    }
}

impl TypeSpanned {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn dummy(ty: Type) -> Self {
        Self {
            ty,
            span: Span::default(),
        }
    }
}

impl Deref for TypeSpanned {
    type Target = Type;
    fn deref(&self) -> &Self::Target {
        &self.ty
    }
}

impl Display for TypeSpanned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}
