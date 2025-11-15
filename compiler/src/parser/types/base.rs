use std::fmt::Display;

use crate::{
    lexer::tokens::Ident,
    parser::{parser::Parser, Parse},
};

use super::PrimitiveType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomType(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType {
    Primitive(PrimitiveType),
    Custom(CustomType),
}

impl BaseType {
    pub fn type_id(&self) -> u8 {
        match self {
            BaseType::Primitive(p) => p.type_id(),
            BaseType::Custom(_) => 0xFF 
        }
    }
}

impl Parse for BaseType {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        if let Some(t) = parser.try_parse::<PrimitiveType>() {
            return Ok(Self::Primitive(t));
        }

        let type_ident = parser.consume::<Ident>()?;
        return Ok(Self::Custom(CustomType(type_ident.value().clone())));
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BaseType::Custom(i) => &i.0,
            BaseType::Primitive(p) => &p.to_string(),
        };

        write!(f, "{s}")
    }
}
