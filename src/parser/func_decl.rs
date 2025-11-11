use super::{
    node::Node,
    parser::Parser,
    statements::BlockStmt,
    types::{Type, TypeSpanned},
    Parse,
};
use crate::{
    aliases::Result,
    generator::compile_trait::Generate,
    lexer::tokens::Ident,
    semantic_analyzer::{analyze_trait::Analyze, scope::ScopeKind},
    t, tt,
};

#[derive(Debug, Clone)]
pub struct Param {
    name: Node<Ident>,
    ttype: TypeSpanned,
}

impl Parse for Param {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        let name = parser.parse_node()?;
        parser.consume::<t!(:)>()?;
        let ttype = parser.parse()?;
        Ok(Self { name, ttype })
    }
}

#[derive(Debug, Clone)]
pub struct FuncDelc {
    name: Node<Ident>,
    params: Vec<Node<Param>>,
    return_type: TypeSpanned,
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

        let mut return_type = TypeSpanned::dummy(Type::void());

        if let tt!(->) = parser.peek()? {
            parser.consume::<t!(->)>()?;
            return_type = parser.parse::<TypeSpanned>()?;
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
        let return_info = builder.register_type(&self.return_type);
        builder.define_func(&self.name, return_info);
        builder.push_scope(ScopeKind::Function(self.name.value().clone()));
        for param in &self.params {
            let param_info = builder.register_type(&param.ttype);
            builder.define_param(&param.name, param_info);
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

impl Generate for FuncDelc {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        let func_label = generator.create_label();
        let id = generator.get_function_mapping(self.name.id());
        generator.register_function(id, func_label);
        if self.name.value() == "main" {
            generator.set_main(id);
        }
        generator.place_label(func_label);
        self.block.generate(generator);
    }
}
