use crate::parser::parse_trait::Parse;
use crate::parser::parser::Parser;
use crate::tt;

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Normal,
    Add,
    Sub,
    Mult,
    Div,
    Mod,
}

impl Parse for AssignOp {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(=) => Self::Normal,
            tt!(+=) => Self::Add,
            tt!(-=) => Self::Sub,
            tt!(*=) => Self::Mult,
            tt!(/=) => Self::Div,
            tt!(%=) => Self::Mod,
            _ => {
                return parser.unexpected("assignment operator");
            }
        };

        parser.discard_next()?;

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::AssignOp;
    use crate::{lexer::Lexer, parser::{parser::Parser, statements::Stmt}, source::SourceFile};
    use std::io::Cursor;

    fn parse_stmt(input: &str) -> Stmt {
        let source = SourceFile::new(Cursor::new(input)).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        parser.parse::<Stmt>().unwrap()
    }


    #[test]
    fn test_assign_op_normal() {
        let stmt = parse_stmt("var = 5;");
        assert!(matches!(stmt, Stmt::AssignStmt(_, AssignOp::Normal, _)));
    }

    #[test]
    fn test_assign_op_add() {
        let stmt = parse_stmt("var += 5;");
        assert!(matches!(stmt, Stmt::AssignStmt(_, AssignOp::Add, _)));
    }

    #[test]
    fn test_assign_op_sub() {
        let stmt = parse_stmt("var -= 5;");
        assert!(matches!(stmt, Stmt::AssignStmt(_, AssignOp::Sub, _)));
    }

    #[test]
    fn test_assign_op_mult() {
        let stmt = parse_stmt("var *= 5;");
        assert!(matches!(stmt, Stmt::AssignStmt(_, AssignOp::Mult, _)));
    }

    #[test]
    fn test_assign_op_div() {
        let stmt = parse_stmt("var /= 5;");
        assert!(matches!(stmt, Stmt::AssignStmt(_, AssignOp::Div, _)));
    }

    #[test]
    fn test_assign_op_mod() {
        let stmt = parse_stmt("var %= 5;");
        assert!(matches!(stmt, Stmt::AssignStmt(_, AssignOp::Mod, _)));
    }
}
