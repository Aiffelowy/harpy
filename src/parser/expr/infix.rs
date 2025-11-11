use std::fmt::Display;

use super::binding_power::Bp;
use crate::generator::compile_trait::Generate;
use crate::generator::instruction::Instruction;
use crate::lexer::span::Span;
use crate::parser::parser::Parser;
use crate::parser::Parse;
use crate::tt;

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOpKind {
    Plus,
    Minus,
    Mult,
    Div,
    And,
    Or,
    Gt,
    Lt,
    Eq,
    GtEq,
    LtEq,
    Neq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfixOp {
    pub op: InfixOpKind,
    span: Span,
}

impl InfixOp {
    pub fn bp(&self) -> Bp {
        match self.op {
            InfixOpKind::Mult | InfixOpKind::Div => (60, 61),
            InfixOpKind::Plus | InfixOpKind::Minus => (50, 51),
            InfixOpKind::GtEq
            | InfixOpKind::LtEq
            | InfixOpKind::Eq
            | InfixOpKind::Lt
            | InfixOpKind::Neq
            | InfixOpKind::Gt => (40, 41),
            InfixOpKind::And => (30, 31),
            InfixOpKind::Or => (20, 21),
        }
        .into()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Display for InfixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InfixOpKind::*;
        let s = match self.op {
            Plus => "add",
            Minus => "subtract",
            Mult => "multiply",
            Div => "divide",
            And => "&&",
            Or => "||",
            Gt => ">",
            Lt => "<",
            Eq => "==",
            GtEq => ">=",
            LtEq => "<=",
            Neq => "!=",
        };

        write!(f, "{s}")
    }
}

impl Parse for InfixOp {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let op = match parser.peek()? {
            tt!(+) => InfixOpKind::Plus,
            tt!(-) => InfixOpKind::Minus,
            tt!(*) => InfixOpKind::Mult,
            tt!(/) => InfixOpKind::Div,
            tt!(&&) => InfixOpKind::And,
            tt!(||) => InfixOpKind::Or,
            tt!(>) => InfixOpKind::Gt,
            tt!(<) => InfixOpKind::Lt,
            tt!(==) => InfixOpKind::Eq,
            tt!(>=) => InfixOpKind::GtEq,
            tt!(<=) => InfixOpKind::LtEq,
            tt!(!=) => InfixOpKind::Neq,
            _ => {
                return parser.unexpected("infix operator");
            }
        };

        let t = parser.discard_next()?;

        Ok(Self { op, span: t.span() })
    }
}

impl Generate for InfixOp {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        use InfixOpKind::*;
        match self.op {
            Plus => generator.push_instruction(Instruction::ADD),
            Minus => generator.push_instruction(Instruction::SUB),
            Mult => generator.push_instruction(Instruction::MUL),
            Div => generator.push_instruction(Instruction::DIV),
            And => generator.push_instruction(Instruction::AND),
            Or => generator.push_instruction(Instruction::OR),
            Gt => generator.push_instruction(Instruction::GT),
            Lt => generator.push_instruction(Instruction::LT),
            Eq => generator.push_instruction(Instruction::EQ),
            GtEq => generator.push_instruction(Instruction::GTE),
            LtEq => generator.push_instruction(Instruction::LTE),
            Neq => generator.push_instruction(Instruction::NEQ),
        }
    }
}
