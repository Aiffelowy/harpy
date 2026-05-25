use crate::{analyzer::analyzer::SemanticDB, err::Kind};

impl Kind {
    pub(super) fn render(&self, db: Option<&SemanticDB>) -> String {
        use Kind::*;
        let db = || db.expect("Attempted to format a late error without a SemanticDB!");
        match self {
            //early
            UnknownToken => "unknown token.".to_string(),
            InvalidInt(e) => format!("invalid integer: `{e}`."),
            InvalidFloat(e) => format!("invalid float: `{e}`."),
            UnclosedStr => "unclosed string".to_string(),
            UnexpectedToken { expected, got } => {
                format!(
                    "unexpected token. expected `{expected}`, got `{}`.",
                    got.kind()
                )
            }
            UnexpectedEof => "unexpected end of file".to_string(),
            MultipleDefaultInSwitch => {
                "multiple default cases in the switch expression.".to_string()
            }
            AlreadyExists { name, .. } => {
                format!("redefinition of the symbol `{name}`.")
            }
            UnknownType(ty) => format!("cannot find type `{ty}` in this scope."),
            UnknownFunction(f) => format!("cannot find function `{f}` in this scope."),
            UnknownSymbol(name) => format!("cannot find value `{}` in this scope.", name),
            ArraySizeInt => "array size must be a constant integer".to_string(),
            MissingMain => "no `main(str) -> void` function found.".to_string(),
            GlobalInFn => "cannot declare a global variable inside a function.".to_string(),

            //late
            ExprNotIter => "expression cannot be iterated through.".to_string(),
            NotCompatible { expected, got } => {
                let db = db();
                format!(
                    "mismatched types. expected `{}`, got `{}`.",
                    expected.format(db),
                    got.format(db)
                )
            }
            IfNotCompatible { expected, got } => {
                let db = db();
                format!(
                    "if-else expression merges into incompatible types. expected `{}`, got `{}`.",
                    expected.format(db),
                    got.format(db)
                )
            }
            InvalidLValue => "cannot assign a value to this expression.".to_string(),
            InvalidArithmetic(ty1, ty2) => {
                let db = db();
                format!(
                    "cannot perform arithmetic on `{}` and `{}`.",
                    ty1.format(db),
                    ty2.format(db)
                )
            }
            InvalidRelational(ty1, ty2) => {
                let db = db();
                format!(
                    "cannot perform boolean operations on `{}` and `{}`.",
                    ty1.format(db),
                    ty2.format(db)
                )
            }
            InvalidPrefix(ty) => {
                let db = db();
                format!("cannot use this prefix on `{}`.", ty.format(db))
            }
            UnknownField(struct_id, field_name) => {
                let db = db();
                let s = db.struct_table.get(*struct_id);
                format!("no field `{}` in struct `{}`.", field_name, s.name)
            }
            NotAStruct(ty) => {
                let db = db();
                format!("`{}` is not a struct.", ty.format(db))
            }
            NotAnArray(ty) => {
                let db = db();
                format!("`{}` is not an array.", ty.format(db))
            }
            MissingField(id, field) => {
                let db = db();
                format!("missing field `{field}` of struct {}.", id.name(db))
            }
            ArgumentCountMismatch(expected, got) => {
                format!("expected {} arguments, got {}.", expected, got)
            }
            NotCallable(ty) => {
                let db = db();
                format!("`{}` is not callable.", ty.format(db))
            }
            ReturnOutsideFn => "return expression outside a function body.".to_string(),
            ContinueOutsideLoop => "continue expression outside a loop body.".to_string(),
            BreakOutsideLoop => "break expression outside a loop body.".to_string(),
            MissingDefaultBranch => {
                "no default branch defined in the switch expression.".to_string()
            }
            TypeAnnotationsNeeded => "cannot infer type. type annotations needed.".to_string(),
            UnexpectedDot => "unexpected dot expression".to_string(),
            RecursiveBox => "cannot box a box".to_string(),
            RecursiveRef => "cannot create a ref to a ref".to_string(),
            BoxedRef => "cannot box a ref".to_string(),
            RefInStruct => "structs cannot hold references".to_string(),

            IO(e) => format!("internal io error: {e}"),
            Custom(e) => e.to_string(),

            _ => format!("{:?}", self),
        }
    }
}
