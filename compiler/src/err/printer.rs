use crate::{analyzer::analyzer::SemanticDB, color::Color, err::HarpyError, source::SourceMap};

pub struct ErrorPrinter;

impl ErrorPrinter {
    pub fn print_diagnostic(map: &SourceMap, db: Option<&SemanticDB>, error: &HarpyError) {
        let msg = error.kind.render(db);

        println!(
            "{}{}Error: {}{}{}",
            Color::Bold,
            Color::Red,
            Color::Reset,
            Color::Bold,
            msg
        );

        let Some(span) = error.span else {
            println!();
            return;
        };

        let source = map.get_source(span.file_id);
        let line_num = span.start.line;
        let col_num = span.start.column;

        let line_text = source
            .get_line(line_num.saturating_sub(1))
            .unwrap_or("")
            .trim_end();

        let span_len = span.end.byte.saturating_sub(span.start.byte);
        let squiggle_len = std::cmp::max(1, span_len);

        let safe_squiggle_len = std::cmp::min(
            squiggle_len,
            line_text.len().saturating_sub(col_num.saturating_sub(1)),
        );

        let line_num_str = line_num.to_string();
        let margin = " ".repeat(line_num_str.len());
        let indent = " ".repeat(col_num.saturating_sub(1));
        let squiggles = "^".repeat(safe_squiggle_len);

        println!(
            "{}  --> {}:{}:{}{}",
            Color::Cyan,
            source.filename,
            line_num,
            col_num,
            Color::Reset
        );

        println!("{} {} |{}", Color::Cyan, margin, Color::Reset);

        println!(
            "{} {} |{} {}",
            Color::Cyan,
            line_num_str,
            Color::Reset,
            line_text
        );

        println!(
            "{} {} |{} {}{}{}{}\n",
            Color::Cyan,
            margin,
            Color::Reset,
            indent,
            Color::Red,
            squiggles,
            Color::Reset
        );
    }

    pub fn print_all(map: &SourceMap, db: Option<&SemanticDB>, errors: &[HarpyError]) {
        for e in errors {
            Self::print_diagnostic(map, db, e);
        }
    }
}
