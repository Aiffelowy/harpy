use super::{node::Node, parser::Parser, statements::BlockStmt, types::Type, Parse};
use crate::{
    aliases::Result,
    lexer::tokens::Ident,
    semantic_analyzer::{
        analyze_trait::Analyze,
        scope::ScopeKind,
        symbol_info::{FunctionInfo, VariableInfo},
    },
    t, tt,
};

#[derive(Debug, Clone)]
pub struct Param {
    name: Node<Ident>,
    ttype: Type,
}

impl Parse for Param {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let name = parser.parse_node()?;
        parser.consume::<t!(:)>()?;
        let ttype = parser.parse::<Type>()?;
        Ok(Self { name, ttype })
    }
}

#[derive(Debug, Clone)]
pub struct FuncDelc {
    name: Node<Ident>,
    params: Vec<Node<Param>>,
    return_type: Type,
    block: BlockStmt,
}

impl FuncDelc {
    fn parse_params(parser: &mut Parser, params: &mut Vec<Node<Param>>) -> Result<()> {
        let first = parser.parse_node::<Param>()?;
        params.push(first);
        loop {
            if let tt!(")") = parser.peek()? {
                break;
            }
            parser.consume::<t!(,)>()?;
            params.push(parser.parse_node::<Param>()?);
        }

        Ok(())
    }
}

impl Parse for FuncDelc {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume::<t!(fn)>()?;
        let name = parser.parse_node()?;
        parser.consume::<t!("(")>()?;
        let mut params = vec![];

        if let tt!(ident) = parser.peek()? {
            match Self::parse_params(parser, &mut params) {
                Ok(()) => (),
                Err(e) => parser.report_error(e, &[tt!(")")])?,
            }
        }

        parser.consume::<t!(")")>()?;

        let mut return_type = Type::void();

        if let tt!(->) = parser.peek()? {
            parser.consume::<t!(->)>()?;
            return_type = parser.parse::<Type>()?;
        }

        let block = parser.parse::<BlockStmt>()?;

        Ok(Self {
            name,
            params,
            return_type,
            block,
        })
    }
}

impl Analyze for FuncDelc {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        builder.define_func(
            &self.name,
            FunctionInfo {
                params: self.params.iter().map(|p| p.ttype.clone()).collect(),
                return_type: self.return_type.clone(),
                locals: vec![],
            },
        );
        builder.push_scope(ScopeKind::Function(self.name.value().clone()));
        for param in &self.params {
            builder.define_var(
                &param.name,
                VariableInfo {
                    ttype: param.ttype.clone(),
                    initialized: true,
                },
            );
        }

        self.block.build(builder);
        builder.pop_scope();
    }

    fn analyze_semantics(&self, analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer) {
        analyzer.enter_scope();
        self.block.analyze_semantics(analyzer);
        analyzer.exit_scope();
    }
}
