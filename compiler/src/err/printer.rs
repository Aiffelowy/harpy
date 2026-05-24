use crate::{
    analyzer::analyzer::SemanticDB,
    color::Color,
    err::HarpyError,
    lexer::span::Span,
    source::{SourceFile, SourceMap},
};

pub struct ErrorPrinter;

impl ErrorPrinter {
    pub fn print_diagnostic(map: &SourceMap, db: Option<&SemanticDB>, error: &HarpyError) {
        let msg = error.kind.render(db);
        Self::print_header(&msg);

        if let Some(span) = error.span {
            Self::print_spanned(map, span);
        } else {
            println!();
        }
    }

    pub fn print_all(map: &SourceMap, db: Option<&SemanticDB>, errors: &[HarpyError]) {
        for e in errors {
            Self::print_diagnostic(map, db, e);
        }
    }

    fn print_header(msg: &str) {
        println!(
            "{}{}Error: {}{}{}",
            Color::Bold,
            Color::Red,
            Color::Reset,
            Color::Bold,
            msg
        );
    }

    fn print_spanned(map: &SourceMap, span: Span) {
        let source = map.get_source(span.file_id);
        let max_line_str = span.end.line.to_string();
        let margin = " ".repeat(max_line_str.len());

        println!(
            "{}  --> {}:{}:{}{}",
            Color::Cyan,
            source.filename,
            span.start.line,
            span.start.column,
            Color::Reset
        );

        println!("{} {} |{}", Color::Cyan, margin, Color::Reset);

        if span.start.line == span.end.line {
            Self::print_single_line(source, span, &max_line_str, &margin);
        } else {
            Self::print_multi_line(source, span, &max_line_str, &margin);
        }
    }

    fn print_single_line(source: &SourceFile, span: Span, max_line_str: &str, margin: &str) {
        let line_num = span.start.line;
        let col = span.start.column;

        let line_text = source
            .get_line(line_num.saturating_sub(1))
            .unwrap_or("")
            .trim_end();

        let span_len = span.end.byte.saturating_sub(span.start.byte);
        let safe_squiggle_len =
            std::cmp::max(1, span_len).min(line_text.len().saturating_sub(col.saturating_sub(1)));

        let line_num_str = format!("{:>width$}", line_num, width = max_line_str.len());
        let indent = " ".repeat(col.saturating_sub(1));
        let squiggles = "^".repeat(safe_squiggle_len);

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

    fn print_multi_line(source: &SourceFile, span: Span, max_line_str: &str, margin: &str) {
        let start_line = span.start.line;
        let end_line = span.end.line;
        let start_col = span.start.column;
        let end_col = span.end.column;

        let first_line_text = source
            .get_line(start_line.saturating_sub(1))
            .unwrap_or("")
            .trim_end();

        let first_line_num = format!("{:>width$}", start_line, width = max_line_str.len());
        let first_indent = " ".repeat(start_col.saturating_sub(1));
        let first_squiggles = "^".repeat(
            first_line_text
                .len()
                .saturating_sub(start_col.saturating_sub(1))
                .max(1),
        );

        println!(
            "{} {} |{} {}",
            Color::Cyan,
            first_line_num,
            Color::Reset,
            first_line_text
        );
        println!(
            "{} {} |{} {}{}{}{}",
            Color::Cyan,
            margin,
            Color::Reset,
            first_indent,
            Color::Red,
            first_squiggles,
            Color::Reset
        );

        if end_line - start_line > 1 {
            println!("{} {} | ...{}", Color::Cyan, margin, Color::Reset);
        }

        let last_line_text = source
            .get_line(end_line.saturating_sub(1))
            .unwrap_or("")
            .trim_end();

        let last_line_num = format!("{:>width$}", end_line, width = max_line_str.len());
        let last_squiggles = "^".repeat(end_col.saturating_sub(1).max(1));

        println!(
            "{} {} |{} {}",
            Color::Cyan,
            last_line_num,
            Color::Reset,
            last_line_text
        );
        println!(
            "{} {} |{} {}{}{}\n",
            Color::Cyan,
            margin,
            Color::Reset,
            Color::Red,
            last_squiggles,
            Color::Reset
        );
    }
}
