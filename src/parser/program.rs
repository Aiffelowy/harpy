use super::{func_decl::FuncDelc, statements::LetStmt, Parse};
use crate::tt;

#[derive(Debug, Clone, PartialEq)]
pub enum SubProgram {
    Let(LetStmt),
    FuncDecl(FuncDelc),
}

impl Parse for SubProgram {
    fn parse(parser: &mut super::parser::Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(let) => Self::Let(parser.parse::<LetStmt>()?),
            tt!(fn) => Self::FuncDecl(parser.parse::<FuncDelc>()?),
            _ => return parser.unexpected("let statement or a function declaration"),
        };

        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    parts: Vec<SubProgram>,
}

impl Parse for Program {
    fn parse(parser: &mut super::parser::Parser) -> crate::aliases::Result<Self> {
        let mut parts = vec![];
        loop {
            if let tt!(eof) = parser.peek()? {
                break;
            }

            parts.push(parser.parse::<SubProgram>()?);
        }

        Ok(Self { parts })
    }
}
