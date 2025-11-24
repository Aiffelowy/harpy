use crate::{
    generator::{compile_trait::Generate, instruction::Instruction},
    get_symbol_mut,
    lexer::tokens::Ident,
    parser::{
        expr::Expr,
        node::Node,
        parse_trait::Parse,
        parser::Parser,
        types::{Type, TypeInner, TypeSpanned},
    },
    semantic_analyzer::{
        analyze_trait::Analyze, err::SemanticError, return_status::ReturnStatus,
        symbol_info::SymbolInfoKind,
    },
    t, tt,
};

#[derive(Debug, Clone)]
pub struct LetStmt {
    var: Node<Ident>,
    ttype: TypeSpanned,
    rhs: Option<Node<Expr>>,
}

impl Parse for LetStmt {
    fn parse(parser: &mut Parser) -> crate::aliases::Result<Self> {
        parser.consume::<t!(let)>()?;
        let var = parser.parse_node::<Ident>()?;

        let mut ttype = TypeSpanned::dummy(Type::unknown());
        if let tt!(:) = parser.peek()? {
            parser.consume::<t!(:)>()?;
            ttype = parser.parse()?;
        }

        let mut rhs = None;
        if let tt!(=) = parser.peek()? {
            parser.consume::<t!(=)>()?;
            rhs = Some(parser.parse_node::<Expr>()?);
        }

        parser.consume::<t!(;)>()?;

        Ok(Self { var, ttype, rhs })
    }
}

impl Analyze for LetStmt {
    fn build(&self, builder: &mut crate::semantic_analyzer::scope_builder::ScopeBuilder) {
        let type_info = builder.register_type(&self.ttype);
        builder.define_var(&self.var, type_info)
    }

    fn analyze_semantics(
        &self,
        analyzer: &mut crate::semantic_analyzer::analyzer::Analyzer,
    ) -> ReturnStatus {
        let Some(rhs) = &self.rhs else {
            return ReturnStatus::Never;
        };

        let Some(expr_type) = analyzer.resolve_expr(rhs) else {
            return ReturnStatus::Never;
        };

        get_symbol_mut!((analyzer, self.var) info {
        if self.ttype.inner == TypeInner::Unknown {
            info.infer_type(&expr_type);
        }

        if !info.ty.assign_compatible(&expr_type.ttype) {
            analyzer.report_semantic_error(
                SemanticError::LetTypeMismatch(info.ty.ttype.clone(), expr_type.clone()),
                rhs.span(),
            );
        }

        if let SymbolInfoKind::Variable(ref mut v) = &mut info.kind {
                v.initialized = true;
            }

        });

        ReturnStatus::Never
    }
}

impl Generate for LetStmt {
    fn generate(&self, generator: &mut crate::generator::generator::Generator) {
        if let Some(expr) = &self.rhs {
            generator.gen_expr(expr);
            let id = generator.get_local_mapping(self.var.id());
            generator.push_instruction(Instruction::STORE_LOCAL(id));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::extensions::SymbolInfoRefExt;
    use crate::{lexer::Lexer, parser::{parser::Parser, types::Type, Parse}, semantic_analyzer::{analyze_trait::Analyze, scope_builder::ScopeBuilder, symbol_info::SymbolInfoKind}, source::SourceFile};

    use super::LetStmt;

    fn parse_let(input: &str) -> LetStmt {
        let source = SourceFile::new(Cursor::new(input)).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        parser.parse::<LetStmt>().unwrap()
    }

    #[test]
    fn test_parse_let_stmt_full() {
        let stmt = parse_let("let var: mut int = (7 + 3) * 4;");
        assert!( stmt.ttype.compatible(&Type::int()) );
        assert!(stmt.rhs.is_some());
        assert_eq!(stmt.var.value(), "var");
    }


    #[test]
    fn test_parse_let_stmt_uninit() {
        let stmt = parse_let("let var;");
        assert_eq!(*stmt.ttype, Type::unknown());
        assert!(stmt.rhs.is_none());
        assert_eq!(stmt.var.value(), "var");
    }


    #[test]
    fn test_parse_let_stmt_without_type() {
        let stmt = parse_let("let var = false;");
        assert_eq!(*stmt.ttype, Type::unknown());
        assert!(stmt.rhs.is_some());
        assert_eq!(stmt.var.value(), "var");
    }


    fn create_analyzer(input: &str) -> (crate::semantic_analyzer::analyzer::Analyzer, LetStmt) {
        let source = SourceFile::new(Cursor::new(input)).unwrap();
        let mut parser = Parser::new(Lexer::new(&source).unwrap());
        let stmt = LetStmt::parse(&mut parser).unwrap();
        let mut sb = ScopeBuilder::new();
        stmt.build(&mut sb);
        let analyzer = sb.into_analyzer();
        (analyzer, stmt)
    }


    #[test]
    fn test_let_semantic_analysis() {
        let (mut analyzer, stmt) = create_analyzer("let var = 45;");
        let status = stmt.analyze_semantics(&mut analyzer);
        assert_eq!(status, crate::semantic_analyzer::return_status::ReturnStatus::Never);
    }

    #[test]
    fn test_let_type_inference() {
        let (mut analyzer, stmt) = create_analyzer("let var = 45;");
        stmt.analyze_semantics(&mut analyzer);
        
        let symbol = analyzer.get_symbol(&stmt.var).unwrap();
        let symbol_info = symbol.get();
        assert_eq!(symbol_info.ty.ttype, Type::int());
        
        if let SymbolInfoKind::Variable(var_info) = &symbol_info.kind {
            assert!(var_info.initialized);
        } else {
            panic!("Expected variable symbol");
        }
    }

    #[test]
    fn test_let_explicit_type_compatibility() {
        let (mut analyzer, stmt) = create_analyzer("let var: int = 45;");

        let status = stmt.analyze_semantics(&mut analyzer);
        assert_eq!(status, crate::semantic_analyzer::return_status::ReturnStatus::Never);
        
        let symbol = analyzer.get_symbol(&stmt.var).unwrap();
        let symbol_info = symbol.get();
        assert_eq!(symbol_info.ty.ttype, Type::int());
    }

    #[test]
    fn test_let_uninitialized() {
        let (mut analyzer, stmt) = create_analyzer("let var;");
        let status = stmt.analyze_semantics(&mut analyzer);
        assert_eq!(status, crate::semantic_analyzer::return_status::ReturnStatus::Never);
        
        let symbol = analyzer.get_symbol(&stmt.var).unwrap();
        let symbol_info = symbol.get();
        
        if let SymbolInfoKind::Variable(var_info) = &symbol_info.kind {
            assert!(!var_info.initialized);
        } else {
            panic!("Expected variable symbol");
        }
    }

    #[test]
    fn test_let_different_types() {
        let test_cases = vec![
            ("let x = true;", Type::bool()),
            ("let y = 3.14;", Type::float()),
            ("let z = \"hello\";", Type::str()),
        ];
        
        for (input, expected_type) in test_cases {
            let (mut analyzer, stmt) = create_analyzer(input);
            stmt.analyze_semantics(&mut analyzer);
            
            let symbol = analyzer.get_symbol(&stmt.var).unwrap();
            let symbol_info = symbol.get();
            assert_eq!(symbol_info.ty.ttype, expected_type, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_let_complex_expressions() {
        let (mut analyzer, stmt) = create_analyzer("let result = 5 + 3 * 2;");
        stmt.analyze_semantics(&mut analyzer);
        
        let symbol = analyzer.get_symbol(&stmt.var).unwrap();
        let symbol_info = symbol.get();
        assert_eq!(symbol_info.ty.ttype, Type::int());
        assert_eq!(stmt.var.value(), "result");
    }
}
