use super::{func_decl::FuncDelc, node::Node, statements::{GlobalStmt}, Parse};
use crate::{
    generator::compile_trait::Generate,
    semantic_analyzer::{analyze_trait::Analyze, return_status::ReturnStatus},
    tt,
};

#[derive(Debug, Clone)]
pub enum SubProgram {
    Global(Node<GlobalStmt>),
    FuncDecl(Node<FuncDelc>),
}

impl Parse for SubProgram {
    fn parse(parser: &mut super::parser::Parser) -> crate::aliases::Result<Self> {
        let s = match parser.peek()? {
            tt!(global) => Self::Global(parser.parse_node::<GlobalStmt>()?),
            tt!(fn) => Self::FuncDecl(parser.parse_node::<FuncDelc>()?),
            _ => return parser.unexpected("global statement or a function declaration"),
        };

        Ok(s)
    }
}

impl Generate for SubProgram {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        match self {
            Self::Global(g) => g.generate(generator),
            Self::FuncDecl(f) => f.generate(generator),
        }
    }
}

#[derive(Debug, Clone)]
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
                SubProgram::Global(global) => global.build(builder),
                SubProgram::FuncDecl(decl) => decl.build(builder),
            }
        }
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) -> ReturnStatus {
        for sub in &self.parts {
            match sub {
                SubProgram::Global(global) => { global.analyze_semantics(analyzer); },
                SubProgram::FuncDecl(decl) => { decl.analyze_semantics(analyzer); },
            }
        }
        
        ReturnStatus::Never
    }
}

impl Generate for Program {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        for sub in &self.parts {
            if let SubProgram::Global(global) = sub {
                global.generate(generator);
            }
        }
        
        generator.push_instruction(crate::generator::instruction::Instruction::CALL(generator.get_main_index()));
        generator.push_instruction(crate::generator::instruction::Instruction::HALT);
        
        for sub in &self.parts {
            if let SubProgram::FuncDecl(func) = sub {
                func.generate(generator);
            }
        }
    }
}
