use std::fmt::Display;

use crate::{
    parser::{parser::Parser, Parse},
    t, tt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Int,
    Str,
    Float,
    Bool,
}

impl Parse for PrimitiveType {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(int) => {
                parser.consume::<t!(int)>()?;
                Self::Int
            }
            tt!(float) => {
                parser.consume::<t!(float)>()?;
                Self::Float
            }
            tt!(str) => {
                parser.consume::<t!(str)>()?;
                Self::Str
            }
            tt!(bool) => {
                parser.consume::<t!(bool)>()?;
                Self::Bool
            }
            _ => {
                return parser.unexpected("primitive type");
            }
        };

        Ok(s)
    }
}

impl PrimitiveType {
    pub fn type_id(&self) -> u8 {
        match self {
            Self::Int => 0x01,
            Self::Float => 0x02,
            Self::Str => 0x03,
            Self::Bool => 0x04
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Str => "str",
            Self::Bool => "bool",
        };

        write!(f, "{s}")
    }
}
