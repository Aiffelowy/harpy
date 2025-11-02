use super::{func_decl::FuncDelc, statements::LetStmt, Parse};
use crate::{
    lexer::span::Span,
    semantic_analyzer::{analyze_trait::Analyze, err::SemanticError},
    tt,
};

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

impl Analyze for Program {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        for sub in &self.parts {
            match sub {
                SubProgram::Let(lets) => lets.build(builder),
                SubProgram::FuncDecl(decl) => decl.build(builder),
            }
        }
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        for sub in &self.parts {
            match sub {
                SubProgram::Let(lets) => lets.analyze_semantics(analyzer),
                SubProgram::FuncDecl(decl) => decl.analyze_semantics(analyzer),
            }
        }

        if !analyzer.main_exists() {
            analyzer.report_semantic_error(SemanticError::MissingMain, Span::default());
        }
    }
}
