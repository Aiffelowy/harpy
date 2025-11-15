use std::fmt::Display;

use crate::generator::compile_trait::Generate;
use crate::lexer::span::Span;
use crate::parser::parser::Parser;
use crate::parser::Parse;
use crate::tt;

use super::binding_power::Bp;

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOpKind {
    Minus,
    Plus,
    Neg,
    Star,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixOp {
    span: Span,
    pub op: PrefixOpKind,
}

impl PrefixOp {
    pub fn bp(&self) -> Bp {
        (0, 70).into()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Display for PrefixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PrefixOpKind::*;
        let s = match self.op {
            Minus => "-",
            Plus => "+",
            Neg => "-",
            Star => "*",
        };

        write!(f, "{s}")
    }
}

impl Parse for PrefixOp {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let op = match parser.peek()? {
            tt!(+) => PrefixOpKind::Plus,
            tt!(-) => PrefixOpKind::Minus,
            tt!(!) => PrefixOpKind::Neg,
            tt!(*) => PrefixOpKind::Star,
            _ => {
                return parser.unexpected("prefix operator");
            }
        };

        let t = parser.discard_next()?;

        Ok(Self { op, span: t.span() })
    }
}

impl Generate for PrefixOp {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        use PrefixOpKind::*;
        match self.op {
            Minus => generator.push_instruction(crate::generator::instruction::Instruction::NEG),
            Plus => {}
            Neg => generator.push_instruction(crate::generator::instruction::Instruction::NOT),
            Star => generator.push_instruction(crate::generator::instruction::Instruction::LOAD),
        }
    }
}
