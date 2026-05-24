use crate::{analyzer::analyzer::SemanticDB, err::Kind};

impl Kind {
    pub(super) fn render(&self, db: Option<&SemanticDB>) -> String {
        let db = || db.expect("Attempted to format a late error without a SemanticDB!");

        match self {
            //early
            Kind::MissingMain => "No \"main()\" function found".to_string(),
            Kind::UnexpectedEof => "Unexpected end of file".to_string(),
            Kind::UnknownSymbol(name) => format!("cannot find value `{}` in this scope", name),
            Kind::UnexpectedToken { expected, got } => {
                format!("expected `{}`, got `{:?}`", expected, got)
            }

            //late
            Kind::IfNotCompatible { expected, got } => {
                let db = db();
                format!(
                    "if-else expression merges into incompatible types. expected: {}, got: {}",
                    expected.format(db),
                    got.format(db)
                )
            }
            Kind::NotCompatible { expected, got } => {
                let db = db();
                format!(
                    "mismatched types. expected: {}, got: {}",
                    expected.format(db),
                    got.format(db)
                )
            }
            Kind::UnknownField(struct_id, field_name) => {
                let db = db();
                let s = db.struct_table.get(*struct_id);
                format!("no field `{}` on struct `{}`", field_name, s.name)
            }

            _ => format!("{:?}", self),
        }
    }
}
