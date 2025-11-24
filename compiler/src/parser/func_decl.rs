use super::{
    node::Node,
    parser::Parser,
    statements::BlockStmt,
    types::{Type, TypeInner, TypeSpanned},
    Parse,
};
use crate::{
    aliases::Result,
    generator::compile_trait::Generate,
    lexer::tokens::Ident,
    semantic_analyzer::{
        analyze_trait::Analyze, err::SemanticError, return_status::ReturnStatus, scope::ScopeKind,
    },
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

    fn analyze_semantics(
        &self,
        analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer,
    ) -> ReturnStatus {
        analyzer.enter_scope();
        let block_status = self.block.analyze_semantics(analyzer);

        if self.return_type.inner != TypeInner::Void && block_status != ReturnStatus::Always {
            analyzer.report_semantic_error(SemanticError::NotAllPathsReturn, self.name.span());
        }

        analyzer.exit_scope();
        block_status
    }
}

impl Generate for FuncDelc {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        let func_label = generator.create_label();
        let id = generator.get_function_mapping(self.name.id());
        generator.register_function(id, func_label);
        generator.place_label(func_label);
        self.block.generate(generator);
        generator.place_ret()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::{lexer::Lexer, parser::parser::Parser, source::SourceFile};
    use super::FuncDelc;

    #[test]
    fn test_simple_function() {
        let source = SourceFile::new(Cursor::new("fn test() -> int { return 42; }")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        let func = parser.parse::<FuncDelc>().unwrap();
        assert_eq!(func.name.value().as_str(), "test");
        assert!(func.params.is_empty());
    }

    #[test]
    fn test_function_with_params() {
        let source = SourceFile::new(Cursor::new("fn add(a: int, b: int) -> int { return a + b; }")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        let func = parser.parse::<FuncDelc>().unwrap();
        assert_eq!(func.name.value().as_str(), "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].name.value().as_str(), "a");
        assert_eq!(func.params[1].name.value().as_str(), "b");
    }

    #[test]
    fn test_void_function() {
        let source = SourceFile::new(Cursor::new("fn print_hello() { }")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        let func = parser.parse::<FuncDelc>().unwrap();
        assert_eq!(func.name.value().as_str(), "print_hello");
        assert!(func.params.is_empty());
    }

    #[test]
    fn test_function_no_params() {
        let source = SourceFile::new(Cursor::new("fn get_answer() -> int { return 42; }")).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        let func = parser.parse::<FuncDelc>().unwrap();
        assert_eq!(func.name.value().as_str(), "get_answer");
        assert!(func.params.is_empty());
    }
}
